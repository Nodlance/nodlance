//! Kubernetes workload discovery and guarded StatefulSet provider.

use nodlance_domain::{
    DatabaseEngine, DiscoveredWorkload, DomainError, ProviderType, ProvisioningPlan,
    ProvisioningStepType,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum K8sProviderError {
    #[error("discovery parsing error: {0}")]
    Discovery(String),
    #[error("manifest generation error: {0}")]
    Manifest(String),
    #[error("kubernetes api error: {0}")]
    Api(String),
}

impl From<K8sProviderError> for DomainError {
    fn from(err: K8sProviderError) -> Self {
        match err {
            K8sProviderError::Discovery(msg) => DomainError::Validation(msg),
            K8sProviderError::Manifest(msg) => DomainError::Validation(msg),
            K8sProviderError::Api(msg) => DomainError::Internal(msg),
        }
    }
}

/// Discovers database workloads from Kubernetes API resources (StatefulSets/Pods/Services).
pub struct K8sDiscoveryScanner;

impl K8sDiscoveryScanner {
    /// Inspects a raw JSON Kubernetes resource (StatefulSet or Pod) and extracts discovered database workload metadata.
    pub fn parse_workload(
        resource: &serde_json::Value,
    ) -> Result<Option<DiscoveredWorkload>, K8sProviderError> {
        let metadata = resource
            .get("metadata")
            .ok_or_else(|| K8sProviderError::Discovery("missing metadata field".to_string()))?;

        let name = metadata
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let namespace = metadata
            .get("namespace")
            .and_then(|ns| ns.as_str())
            .map(|s| s.to_string());

        let labels = metadata.get("labels").and_then(|l| l.as_object());

        // Extract engine from labels or name
        let detected_engine = labels
            .and_then(|l| {
                l.get("app.kubernetes.io/name")
                    .or_else(|| l.get("app"))
                    .and_then(|v| v.as_str())
            })
            .or_else(|| {
                if name.contains("postgres") {
                    Some("postgresql")
                } else if name.contains("mysql") {
                    Some("mysql")
                } else if name.contains("mariadb") {
                    Some("mariadb")
                } else {
                    None
                }
            });

        let engine = match detected_engine {
            Some(s) if s.contains("postgres") => DatabaseEngine::PostgreSql,
            Some(s) if s.contains("mysql") => DatabaseEngine::MySql,
            Some(s) if s.contains("mariadb") => DatabaseEngine::MariaDb,
            _ => return Ok(None),
        };

        // Extract container image and detected version
        let spec = resource.get("spec");
        let containers = spec
            .and_then(|s| s.get("template"))
            .and_then(|t| t.get("spec"))
            .and_then(|ts| ts.get("containers"))
            .or_else(|| spec.and_then(|s| s.get("containers")))
            .and_then(|c| c.as_array());

        let mut detected_version = None;
        let mut endpoints = Vec::new();

        if let Some(c_list) = containers {
            if let Some(first_container) = c_list.first() {
                if let Some(image) = first_container.get("image").and_then(|i| i.as_str()) {
                    if let Some((_, tag)) = image.split_once(':') {
                        detected_version = Some(tag.to_string());
                    }
                }
            }
        }

        let status_str = resource
            .get("status")
            .and_then(|s| s.get("phase"))
            .and_then(|p| p.as_str())
            .unwrap_or("Active")
            .to_string();

        let ns_str = namespace.clone().unwrap_or_else(|| "default".to_string());
        endpoints.push(format!("{name}.{ns_str}.svc.cluster.local"));

        Ok(Some(DiscoveredWorkload {
            id: Uuid::now_v7(),
            provider_type: ProviderType::Kubernetes,
            namespace,
            workload_name: name,
            engine,
            detected_version,
            endpoints,
            status: status_str,
            details: Some(resource.clone()),
            last_seen_at: chrono::Utc::now(),
        }))
    }
}

/// Generates declarative, hardened Kubernetes manifests for database workloads.
pub struct K8sManifestGenerator;

impl K8sManifestGenerator {
    pub fn generate_statefulset(plan: &ProvisioningPlan) -> serde_json::Value {
        let name = format!("{}-{}", plan.engine.as_str(), plan.id);
        let namespace = plan
            .config
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string());

        let (default_port, image) = match plan.engine {
            DatabaseEngine::PostgreSql => (5432, "postgres:16-alpine"),
            DatabaseEngine::MySql => (3306, "mysql:8.0"),
            DatabaseEngine::MariaDb => (3306, "mariadb:11.4"),
        };

        let port = plan.config.port.unwrap_or(default_port);
        let cpu_limit = plan
            .config
            .cpu_limit_cores
            .map(|c| format!("{c}"))
            .unwrap_or_else(|| "2".to_string());
        let memory_limit = plan
            .config
            .memory_limit_mb
            .map(|m| format!("{m}Mi"))
            .unwrap_or_else(|| "2048Mi".to_string());

        serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "StatefulSet",
            "metadata": {
                "name": name,
                "namespace": namespace,
                "labels": {
                    "app.kubernetes.io/name": plan.engine.as_str(),
                    "app.kubernetes.io/managed-by": "nodlance",
                    "nodlance.io/plan-id": plan.id.to_string()
                }
            },
            "spec": {
                "serviceName": format!("{name}-headless"),
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app.kubernetes.io/name": plan.engine.as_str(),
                        "nodlance.io/plan-id": plan.id.to_string()
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app.kubernetes.io/name": plan.engine.as_str(),
                            "nodlance.io/plan-id": plan.id.to_string()
                        }
                    },
                    "spec": {
                        "securityContext": {
                            "runAsNonRoot": true,
                            "runAsUser": 999,
                            "fsGroup": 999
                        },
                        "containers": [{
                            "name": plan.engine.as_str(),
                            "image": plan.config.container_image.as_deref().unwrap_or(image),
                            "ports": [{
                                "containerPort": port,
                                "name": "db-port"
                            }],
                            "resources": {
                                "limits": {
                                    "cpu": cpu_limit,
                                    "memory": memory_limit
                                },
                                "requests": {
                                    "cpu": "250m",
                                    "memory": "512Mi"
                                }
                            },
                            "volumeMounts": [{
                                "name": "data",
                                "mountPath": if plan.engine == DatabaseEngine::PostgreSql {
                                    "/var/lib/postgresql/data"
                                } else {
                                    "/var/lib/mysql"
                                }
                            }]
                        }]
                    }
                },
                "volumeClaimTemplates": [{
                    "metadata": {
                        "name": "data"
                    },
                    "spec": {
                        "accessModes": ["ReadWriteOnce"],
                        "resources": {
                            "requests": {
                                "storage": "10Gi"
                            }
                        }
                    }
                }]
            }
        })
    }

    pub fn generate_headless_service(plan: &ProvisioningPlan) -> serde_json::Value {
        let name = format!("{}-{}-headless", plan.engine.as_str(), plan.id);
        let namespace = plan
            .config
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let port = plan.config.port.unwrap_or(5432);

        serde_json::json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": name,
                "namespace": namespace,
                "labels": {
                    "nodlance.io/plan-id": plan.id.to_string()
                }
            },
            "spec": {
                "clusterIP": "None",
                "ports": [{
                    "port": port,
                    "targetPort": port,
                    "name": "db-port"
                }],
                "selector": {
                    "nodlance.io/plan-id": plan.id.to_string()
                }
            }
        })
    }
}

/// Rollback actions for Kubernetes resources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum K8sRollbackAction {
    DeleteStatefulSet { name: String, namespace: String },
    DeleteService { name: String, namespace: String },
    DeletePersistentVolumeClaim { name: String, namespace: String },
}

pub struct K8sRollbackPlanner;

impl K8sRollbackPlanner {
    pub fn plan_rollback(
        plan: &ProvisioningPlan,
        completed_steps: &[ProvisioningStepType],
    ) -> Vec<K8sRollbackAction> {
        let mut actions = Vec::new();
        let name = format!("{}-{}", plan.engine.as_str(), plan.id);
        let namespace = plan
            .config
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string());

        for step in completed_steps.iter().rev() {
            match step {
                ProvisioningStepType::StartService | ProvisioningStepType::VerifyReadiness => {
                    actions.push(K8sRollbackAction::DeleteStatefulSet {
                        name: name.clone(),
                        namespace: namespace.clone(),
                    });
                    actions.push(K8sRollbackAction::DeleteService {
                        name: format!("{name}-headless"),
                        namespace: namespace.clone(),
                    });
                }
                ProvisioningStepType::PrepareStorage | ProvisioningStepType::InitializeDatabase => {
                    actions.push(K8sRollbackAction::DeletePersistentVolumeClaim {
                        name: format!("data-{name}-0"),
                        namespace: namespace.clone(),
                    });
                }
                _ => {}
            }
        }

        actions.dedup();
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nodlance_domain::{ProviderType, ProvisioningConfig, ProvisioningStatus};

    #[test]
    fn test_parse_discovered_postgres_statefulset() {
        let raw = serde_json::json!({
            "metadata": {
                "name": "production-pg-cluster",
                "namespace": "databases",
                "labels": {
                    "app.kubernetes.io/name": "postgresql"
                }
            },
            "spec": {
                "template": {
                    "spec": {
                        "containers": [{
                            "name": "postgres",
                            "image": "postgres:16.2-alpine"
                        }]
                    }
                }
            },
            "status": {
                "phase": "Running"
            }
        });

        let workload = K8sDiscoveryScanner::parse_workload(&raw)
            .expect("parse workload")
            .expect("should find workload");

        assert_eq!(workload.workload_name, "production-pg-cluster");
        assert_eq!(workload.namespace, Some("databases".to_string()));
        assert_eq!(workload.engine, DatabaseEngine::PostgreSql);
        assert_eq!(workload.detected_version, Some("16.2-alpine".to_string()));
        assert_eq!(
            workload.endpoints[0],
            "production-pg-cluster.databases.svc.cluster.local"
        );
    }

    #[test]
    fn test_generate_statefulset_manifest() {
        let plan = ProvisioningPlan {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: None,
            provider_type: ProviderType::Kubernetes,
            engine: DatabaseEngine::PostgreSql,
            engine_version: "16.1".to_string(),
            config: ProvisioningConfig {
                port: Some(5432),
                data_directory: None,
                max_connections: None,
                shared_buffers_mb: None,
                memory_limit_mb: Some(4096),
                cpu_limit_cores: Some(4.0),
                container_image: None,
                namespace: Some("staging".to_string()),
                extra_params: None,
            },
            status: ProvisioningStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let manifest = K8sManifestGenerator::generate_statefulset(&plan);
        assert_eq!(manifest["apiVersion"], "apps/v1");
        assert_eq!(manifest["kind"], "StatefulSet");
        assert_eq!(manifest["metadata"]["namespace"], "staging");
        assert_eq!(
            manifest["spec"]["template"]["spec"]["securityContext"]["runAsNonRoot"],
            true
        );
    }

    #[test]
    fn test_k8s_rollback_ordering() {
        let plan = ProvisioningPlan {
            id: Uuid::now_v7(),
            cluster_id: Uuid::now_v7(),
            host_id: None,
            provider_type: ProviderType::Kubernetes,
            engine: DatabaseEngine::PostgreSql,
            engine_version: "16.1".to_string(),
            config: ProvisioningConfig::default(),
            status: ProvisioningStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let steps = [
            ProvisioningStepType::PreflightCheck,
            ProvisioningStepType::PrepareStorage,
            ProvisioningStepType::StartService,
        ];

        let rollback = K8sRollbackPlanner::plan_rollback(&plan, &steps);
        assert_eq!(rollback.len(), 3);
        assert!(matches!(
            rollback[0],
            K8sRollbackAction::DeleteStatefulSet { .. }
        ));
        assert!(matches!(
            rollback[1],
            K8sRollbackAction::DeleteService { .. }
        ));
        assert!(matches!(
            rollback[2],
            K8sRollbackAction::DeletePersistentVolumeClaim { .. }
        ));
    }
}
