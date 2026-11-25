<script lang="ts">
	let { ruta, clase = 'h-5 w-5' }: { ruta: string; clase?: string } = $props();
</script>

<svg
	class={clase}
	fill="none"
	viewBox="0 0 24 24"
	stroke-width="1.7"
	stroke="currentColor"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#if ruta === '/'}
		<path d="M2.25 3h1.4l.9 3.75m0 0 1.95 8.25h9.9l1.95-8.25H4.55Z" />
		<path d="M8.25 19.5a1.125 1.125 0 1 0 0-2.25 1.125 1.125 0 0 0 0 2.25Z" />
		<path d="M16.5 19.5a1.125 1.125 0 1 0 0-2.25 1.125 1.125 0 0 0 0 2.25Z" />
	{:else if ruta === '/inventario'}
		<path d="m12 3 8.25 4.5v9L12 21l-8.25-4.5v-9L12 3Z" />
		<path d="m3.75 7.5 8.25 4.5 8.25-4.5M12 12v9" />
	{:else if ruta === '/conciliacion'}
		<path d="M7.5 7.5h12m0 0-3-3m3 3-3 3" />
		<path d="M16.5 16.5h-12m0 0 3 3m-3-3 3-3" />
	{:else if ruta === '/cierre'}
		<path d="M3.75 6.75h16.5v10.5H3.75z" />
		<path d="M3.75 9.75h16.5M7.5 14.25h3.75" />
	{:else if ruta === '/configuracion'}
		<path d="M10.5 4.5h3l.45 2.1a7.2 7.2 0 0 1 1.72 1l2.02-.72 1.5 2.6-1.58 1.47a7.4 7.4 0 0 1 0 2l1.58 1.48-1.5 2.6-2.02-.72a7.2 7.2 0 0 1-1.72 1l-.45 2.1h-3l-.45-2.1a7.2 7.2 0 0 1-1.72-1l-2.02.72-1.5-2.6 1.58-1.47a7.4 7.4 0 0 1 0-2L5.81 9.48l1.5-2.6 2.02.72a7.2 7.2 0 0 1 1.72-1l.45-2.1Z" />
		<circle cx="12" cy="12" r="2.25" />
	{:else if ruta === '/devoluciones'}
		<path d="M9 15 4.5 10.5 9 6" />
		<path d="M4.5 10.5H15a4.5 4.5 0 0 1 0 9h-3" />
	{:else}
		<circle cx="12" cy="12" r="8.25" />
	{/if}
</svg>
