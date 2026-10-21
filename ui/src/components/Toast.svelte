<script>
  import { toasts } from '../lib/stores.js';
</script>

<div class="toast-container" id="toastContainer">
  {#each $toasts as t (t.id)}
    <div class="toast {t.type}">
      <div class="toast-content">{t.message}</div>
    </div>
  {/each}
</div>
