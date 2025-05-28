<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card";

  export let url: string;
  export let name: string;
  export let description: string;
  export let icon_url: string;

  export let width: string = "200px";
</script>

<Card.Root
  onclick={() => {
    document.location = url;
  }}
  style="width: {width}; cursor: pointer;"
>
  <Card.Header>
    <Card.Title>{name}</Card.Title>
  </Card.Header>
  <Card.Content>
    <img src={icon_url} alt={name} {width} />
    <p>
      {description}
    </p>
  </Card.Content>
</Card.Root>
