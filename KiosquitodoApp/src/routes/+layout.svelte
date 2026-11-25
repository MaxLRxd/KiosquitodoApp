<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { Snippet } from 'svelte';
	import Bloqueo from '$lib/components/Bloqueo.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import { obtenerConfiguracion, pinConfigurado } from '$lib/api';
	import { appState, notificar } from '$lib/stores/app.svelte';
	import { MODULOS } from '$lib/utils/teclas';

	let { children }: { children: Snippet } = $props();

	let hora = $state(new Date());
	let ultimaActividad = Date.now();
	let temporizador: ReturnType<typeof setTimeout> | null = null;

	const eventosActividad = ['keydown', 'mousedown', 'pointermove', 'wheel', 'touchstart'];

	function armarTemporizador(): void {
		if (temporizador) clearTimeout(temporizador);
		temporizador = null;

		const minutos = appState.inactividadMinutos;
		if (!appState.pinConfigurado || minutos <= 0 || appState.bloqueado) return;

		const restante = Math.max(0, minutos * 60_000 - (Date.now() - ultimaActividad));
		temporizador = setTimeout(() => {
			if (appState.pinConfigurado && appState.inactividadMinutos > 0) {
				appState.bloqueado = true;
			}
		}, restante);
	}

	function registrarActividad(evento?: Event): void {
		// El evento propio se dispara al desbloquear: actualiza la marca aunque
		// Svelte todavía no haya aplicado `bloqueado = false`.
		if (evento?.type !== 'kiosco:actividad' && appState.bloqueado) return;
		ultimaActividad = Date.now();
		if (!appState.bloqueado) armarTemporizador();
	}

	$effect(() => {
		const timer = setInterval(() => (hora = new Date()), 1000);
		return () => clearInterval(timer);
	});

	// Se rearma cuando cambian el PIN, los minutos de inactividad o el bloqueo.
	$effect(() => {
		const _minutos = appState.inactividadMinutos;
		const _pin = appState.pinConfigurado;
		const _bloqueado = appState.bloqueado;
		void _minutos;
		void _pin;
		void _bloqueado;
		armarTemporizador();
		return () => {
			if (temporizador) clearTimeout(temporizador);
		};
	});

	onMount(() => {
		void cargarEstadoBloqueo();
		for (const evento of eventosActividad) {
			window.addEventListener(evento, registrarActividad, { passive: true });
		}
		window.addEventListener('kiosco:actividad', registrarActividad);
		return () => {
			for (const evento of eventosActividad) {
				window.removeEventListener(evento, registrarActividad);
			}
			window.removeEventListener('kiosco:actividad', registrarActividad);
			if (temporizador) clearTimeout(temporizador);
		};
	});

	async function cargarEstadoBloqueo(): Promise<void> {
		try {
			const [preferencias, hayPin] = await Promise.all([
				obtenerConfiguracion(),
				pinConfigurado()
			]);
			appState.inactividadMinutos = preferencias.inactividad_minutos;
			appState.pinConfigurado = hayPin;
			ultimaActividad = Date.now();
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}
	}

	function esActiva(ruta: string): boolean {
		return page.url.pathname === ruta;
	}

	function manejarTeclado(evento: KeyboardEvent) {
		if (appState.bloqueado) return;
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

{#if appState.bloqueado}
	<Bloqueo />
{/if}

<Toast />