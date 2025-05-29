<script lang="ts">
  let inp = "00000000";
  let diff = "";

  const diff_storage = localStorage.getItem("diff");
  if (diff_storage) {
    diff = diff_storage;
  }

  $: localStorage.setItem("diff", diff);
</script>

<textarea bind:value={inp} placeholder="Input"></textarea><br />
added value: {(parseInt(inp, 16) + parseInt(diff, 16)).toString(16)}<br />

<textarea bind:value={diff} placeholder="A value to add"></textarea>
