<script lang="ts">
  import { ghs_demangle } from "$lib";

  let inp = "a__2ABFv";
</script>

<input type="text" bind:value={inp} placeholder="Input" /><br />
Demangled: {ghs_demangle(inp)}<br />
