<script lang="ts">
  import "../app.css";
  import AppsBar from "$component/AppsBar.svelte";
  import { ModeWatcher } from "mode-watcher";
  import ThemeSwitcher from "$component/ThemeSwitcher.svelte";

  let { children } = $props();
</script>

<ModeWatcher />
<div class="flex flex-col h-full">
  <AppsBar></AppsBar>
  <div style:flex-grow="1">
    {@render children()}
  </div>
</div>
