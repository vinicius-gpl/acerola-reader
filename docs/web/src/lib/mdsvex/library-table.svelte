<script lang="ts">
	import * as Table from '$lib/components/ui/table/index.js';

	type Library = {
		name: string;
		url: string;
		license: string;
		note?: string;
	};

	let {
		libraries,
		nameLabel = 'Library',
		licenseLabel = 'License'
	}: { libraries: Library[]; nameLabel?: string; licenseLabel?: string } = $props();
</script>

<Table.Root>
	<Table.Header>
		<Table.Row class="hover:bg-transparent">
			<Table.Head>{nameLabel}</Table.Head>
			<Table.Head>{licenseLabel}</Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each libraries as lib (lib.name)}
			<Table.Row>
				<Table.Cell>
					<a
						href={lib.url}
						target="_blank"
						rel="noopener noreferrer"
						class="font-medium text-primary underline underline-offset-4 hover:text-primary/80"
					>
						{lib.name}
					</a>
				</Table.Cell>
				<Table.Cell class="text-muted-foreground">
					{lib.license}{#if lib.note}<sup class="ml-0.5">{lib.note}</sup>{/if}
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
