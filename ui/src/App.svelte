<script>
  import { onMount } from 'svelte';
  import { activeView, fetchLiveClusters, hosts } from './lib/stores.js';
  import TopNav from './components/TopNav.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import FleetView from './components/FleetView.svelte';
  import TopologyView from './components/TopologyView.svelte';
  import HaRecoveryView from './components/HaRecoveryView.svelte';
  import LoadBalancersView from './components/LoadBalancersView.svelte';
  import BackupsView from './components/BackupsView.svelte';
  import ProvisioningView from './components/ProvisioningView.svelte';
  import DiscoveryView from './components/DiscoveryView.svelte';
  import HostsView from './components/HostsView.svelte';
  import AlarmsView from './components/AlarmsView.svelte';
  import AuditView from './components/AuditView.svelte';
  import Modals from './components/Modals.svelte';
  import Toast from './components/Toast.svelte';

  onMount(() => {
    fetchLiveClusters();

    // Heartbeat ticker
    const timer = setInterval(() => {
      hosts.update(list => list.map(h => ({
        ...h,
        heartbeatSec: Math.floor(Math.random() * 3) + 1
      })));
    }, 3000);

    return () => clearInterval(timer);
  });
</script>

<TopNav />

<div class="app-layout">
  <Sidebar />

  <main class="content-area">
    {#if $activeView === 'fleet'}
      <FleetView />
    {:else if $activeView === 'topology'}
      <TopologyView />
    {:else if $activeView === 'load-balancers'}
      <LoadBalancersView />
    {:else if $activeView === 'ha-recovery'}
      <HaRecoveryView />
    {:else if $activeView === 'backups'}
      <BackupsView />
    {:else if $activeView === 'provisioning'}
      <ProvisioningView />
    {:else if $activeView === 'discovery'}
      <DiscoveryView />
    {:else if $activeView === 'hosts'}
      <HostsView />
    {:else if $activeView === 'alarms'}
      <AlarmsView />
    {:else if $activeView === 'audit'}
      <AuditView />
    {/if}
  </main>
</div>

<Modals />
<Toast />
