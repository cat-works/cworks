<script lang="ts">
  import OverrayableTextarea from "./OverrayableTextarea.svelte";
  import Icon from "@iconify/svelte";

  // bind
  export let value = "";

  // imcoming
  export let copy_buffer: string;

  // outgoing
  export let selection_start = 0;
  export let selection_end = 0;
</script>

<div style="height: 100%; width: 100%; display: flex; flex-direction: column;">
  <div style="display: flex;">
    <slot name="configurations" />

    <Icon
      on:click={() => {
        selection_start = 0;
        selection_end = value.length;
      }}
      icon="mdi:select_all"
    />
    <Icon
      on:click={() => {
        navigator.clipboard.writeText(copy_buffer);
      }}
      icon="mdi:content_copy"
    />
  </div>
  <OverrayableTextarea
    style="flex: 1;"
    bind:value
    bind:selection_start
    bind:selection_end
  >
    <slot name="overlay" />
  </OverrayableTextarea>
</div>
