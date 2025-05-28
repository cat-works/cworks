<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import IconText from "$component/LeadingIcon.svelte";
  import { contacts, organizations } from "$data/introducation";

  import Organization from "./Organization.svelte";
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>連絡先</Card.Title>
  </Card.Header>

  <Card.Content>
    {#each contacts as contact}
      <IconText icon={contact.icon}>
        <svelte:fragment>
          {contact.name}:&nbsp;
          {#if contact.href}
            <a href={contact.href} class="l">{contact.link_text}</a>
          {/if}
          {#if contact.text}
            {contact.text}
          {/if}
        </svelte:fragment>
      </IconText>
    {/each}
  </Card.Content>
</Card.Root>

<Card.Root>
  <Card.Header>
    <Card.Title>GitHub Organizations</Card.Title>
  </Card.Header>
  <Card.Content>
    <div style:display="flex">
      {#each organizations as organization}
        <Organization {...organization} />
      {/each}
    </div>
  </Card.Content>
</Card.Root>
