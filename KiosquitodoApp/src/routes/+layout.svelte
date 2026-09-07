<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { Snippet } from 'svelte';
	import Toast from '$lib/components/Toast.svelte';
	import { appState } from '$lib/stores/app.svelte';
	import { MODULOS } from '$lib/utils/teclas';

	let { children }: { children: Snippet } = $props();

	let hora = $state(new Date());

	$effect(() => {
		const timer = setInterval(() => (hora = new Date()), 1000);
		return () => clearInterval(timer);
	});

	function esActiva(ruta: string): boolean {
		return page.url.pathname === ruta;
	}

	function manejarTeclado(evento: KeyboardEvent) {
		const modulo = MODULOS.find((m) => m.tecla === evento.key);
		if (modulo) {
			evento.preventDefault();
			goto(modulo.ruta);
		}
	}

	$effect(() => {
		window.addEventListener('keydown', manejarTeclado);
		return () => window.removeEventListener('keydown', manejarTeclado);
	});
</script>

<div class="flex h-screen overflow-hidden">
	<nav class="w-52 shrink-0 border-r border-slate-200 bg-white">
		<div class="px-4 py-4 text-lg font-bold tracking-tight">Kiosquitodo</div>
		<ul class="space-y-1 px-2">
			{#each MODULOS as modulo (modulo.ruta)}
				<li>
					<a
						href={modulo.ruta}
						onclick={(e) => {
							e.preventDefault();
							goto(modulo.ruta);
						}}
						class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm {esActiva(modulo.ruta)
							? 'bg-blue-600 text-white'
							: 'text-slate-700 hover:bg-slate-100'}"
					>
						<span class="w-6 text-xs opacity-60">{modulo.tecla}</span>
						{modulo.nombre}
					</a>
				</li>
			{/each}
		</ul>
	</nav>

	<div class="flex min-w-0 flex-1 flex-col">
		<main class="flex-1 overflow-auto">
			{@render children()}
		</main>
		<footer
			class="flex items-center justify-between border-t border-slate-200 bg-white px-4 py-2 text-xs text-slate-600"
		>
			<span>
				{#if appState.sincronizando}
					<span class="text-amber-500">● Sincronizando MP…</span>
				{:else}
					<span class={appState.conectado ? 'text-green-600' : 'text-red-600'}>
						● {appState.conectado ? 'Conectado' : 'Sin conexión'}
					</span>
				{/if}
			</span>
			<span>{hora.toLocaleTimeString('es-AR')}</span>
		</footer>
	</div>
</div>

<Toast />