<script lang="ts">
  export let value: string;
  export let style = "";
  export let font = "monospace";
  export let height: string;
  export let width: string;

  export let textarea: HTMLTextAreaElement | null = null;

  if (font == "monospace") {
    font = "'Fira Code Light', monospace";
  }
</script>

<textarea
  bind:value
  bind:this={textarea}
  style:resize="true"
  style="height: {height}; width: {width}; {style}; margin: 0px; padding: 16px; height: 100%; width: 100%; font-family: {font};"
  class="sized-textfield"
></textarea>
