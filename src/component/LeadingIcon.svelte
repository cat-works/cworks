<script lang="ts">
  import Icon from "@iconify/svelte";

  export let icon: string;
</script>

<div class="flex">
  <Icon icon="mdi:{icon}" />
  &nbsp;
  <slot />
</div>
