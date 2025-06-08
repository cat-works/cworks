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
      <!-- <Menubar.Item href="/conv/cafecode">CafeCode</Menubar.Item> -->
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
    <Menubar.Trigger>Cat OS</Menubar.Trigger>
    <Menubar.Content>
      <Menubar.Item href="/debug/debug">Shell</Menubar.Item>
    </Menubar.Content>
  </Menubar.Menu>
</Menubar.Root>
