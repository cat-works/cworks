<script lang="ts">
  import ThemeSwitcher from "./ThemeSwitcher.svelte";
  import * as Menubar from "$lib/components/ui/menubar";
</script>

<Menubar.Root>
  <ThemeSwitcher />
  <Menubar.Menu>
    <Menubar.Trigger>About</Menubar.Trigger>
    <Menubar.Content>
      <Menubar.Item href="/about">About Me</Menubar.Item>
    </Menubar.Content>
  </Menubar.Menu>
  <Menubar.Menu>
    <Menubar.Trigger>Converters</Menubar.Trigger>
    <Menubar.Content>
      <Menubar.Item href="/conv/cafecode">CafeCode</Menubar.Item>
      <Menubar.Item href="/conv/ppc-disasm">PowerPC Disasm</Menubar.Item>
    </Menubar.Content>
  </Menubar.Menu>
  <Menubar.Menu>
    <Menubar.Trigger>Tools</Menubar.Trigger>
    <Menubar.Content>
      <Menubar.Item href="/tools/png-extractor">PNG Extractor</Menubar.Item>
      <Menubar.Item href="/tools/ghs-demangler">GHS Demangler</Menubar.Item>
      <Menubar.Item href="/tools/hex-adder">HEX Adder</Menubar.Item>
    </Menubar.Content>
  </Menubar.Menu>
  <Menubar.Menu>
    <Menubar.Trigger>Other</Menubar.Trigger>
    <Menubar.Content>
      <Menubar.Item href="/debug/debug">Debug</Menubar.Item>
    </Menubar.Content>
  </Menubar.Menu>
</Menubar.Root>

<!--
 #if active == "Cafecode Converter"}
      <CafeCode />
    {:else if active == "PowerPC Disassembler"}
      <PPCDisassembler />
    {:else if active == "PNG Extractor"}
      <PNGExtractor />
    {:else if active == "HEX Adder"}
      <HexAdder />
    {:else if active == "syoch"}
      <Introducation />
    {:else if active == "Calculator"}
      <Calc />
    {:else if active == "GHS Demangle"}
      <GHSDemangle />
    {:else if active == "Debug"}
      <Debug />
    {/if}
    --->
