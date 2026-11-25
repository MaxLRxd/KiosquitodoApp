<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { Snippet } from 'svelte';
	import Actualizacion from '$lib/components/Actualizacion.svelte';
	import Bloqueo from '$lib/components/Bloqueo.svelte';
	import IconoModulo from '$lib/components/IconoModulo.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import { obtenerConfiguracion, pinConfigurado, buscarProductoPorBarcode } from '$lib/api';
	import { appState, mensajeError, notificar } from '$lib/stores/app.svelte';
	import { MODULOS } from '$lib/utils/teclas';
	import { crearListenerEscaneo } from '$lib/utils/scanner';
	import { reproducirSonido } from '$lib/utils/sonido';
	import { formatearPesos } from '$lib/utils/moneda';
	import type { Producto } from '$lib/types';

	let { children }: { children: Snippet } = $props();

	let hora = $state(new Date());
	let productoConsulta = $state<Producto | null>(null);
	let temporizadorConsulta: ReturnType<typeof setTimeout> | null = null;

	const moduloActual = $derived(
		MODULOS.find((modulo) => modulo.ruta === page.url.pathname) ?? MODULOS[0]
	);
	const fechaLarga = $derived(
		hora.toLocaleDateString('es-AR', { weekday: 'long', day: 'numeric', month: 'long' })
	);
	let ultimaActividad = Date.now();
	let temporizador: ReturnType<typeof setTimeout> | null = null;

	const eventosActividad = ['keydown', 'mousedown', 'wheel', 'touchstart'];

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
		const offEscaneo = crearListenerEscaneo((codigo) => {
			// En el POS el propio módulo agrega el producto al ticket.
			if (page.url.pathname === '/' || appState.bloqueado) return;
			void consultarPrecio(codigo);
		});
		for (const evento of eventosActividad) {
			window.addEventListener(evento, registrarActividad, { passive: true });
		}
		window.addEventListener('kiosco:actividad', registrarActividad);
		return () => {
			offEscaneo();
			for (const evento of eventosActividad) {
				window.removeEventListener(evento, registrarActividad);
			}
			window.removeEventListener('kiosco:actividad', registrarActividad);
			if (temporizador) clearTimeout(temporizador);
			if (temporizadorConsulta) clearTimeout(temporizadorConsulta);
		};
	});

	async function consultarPrecio(codigo: string): Promise<void> {
		try {
			const producto = await buscarProductoPorBarcode(codigo);
			if (!producto) {
				notificar(`Producto no encontrado: ${codigo}`, 'warn');
				reproducirSonido('scan_error');
				return;
			}
			productoConsulta = producto;
			reproducirSonido('scan_ok');
			if (temporizadorConsulta) clearTimeout(temporizadorConsulta);
			temporizadorConsulta = setTimeout(() => {
				productoConsulta = null;
				temporizadorConsulta = null;
			}, 3000);
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		}
	}

	function cerrarConsulta(): void {
		if (temporizadorConsulta) clearTimeout(temporizadorConsulta);
		temporizadorConsulta = null;
		productoConsulta = null;
	}

	async function cargarEstadoBloqueo(): Promise<void> {
		try {
			const [preferencias, hayPin] = await Promise.all([
				obtenerConfiguracion(),
				pinConfigurado()
			]);
			appState.inactividadMinutos = preferencias.inactividad_minutos;
			appState.pinConfigurado = hayPin;
			// Con PIN configurado, la app arranca bloqueada: nadie opera sin
			// identificarse. La primera vez (sin PIN) no se pide nada.
			appState.bloqueado = hayPin;
			ultimaActividad = Date.now();
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
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

<div class="flex h-screen overflow-hidden bg-slate-50">
	<aside class="flex w-64 shrink-0 flex-col bg-slate-950 text-slate-300">
		<div class="flex h-20 items-center gap-3 px-5">
			<div
				class="grid h-10 w-10 place-items-center rounded-xl bg-gradient-to-br from-indigo-500 to-sky-500 text-lg font-black text-white shadow-lg shadow-indigo-950/50"
			>
				K
			</div>
			<div class="min-w-0">
				<p class="truncate text-base font-bold tracking-tight text-white">Kiosquitodo</p>
				<p class="truncate text-[10px] font-semibold uppercase tracking-[0.18em] text-slate-500">
					Gestión de kiosco
				</p>
			</div>
		</div>

		<nav class="flex-1 overflow-y-auto px-3 py-2">
			<p class="px-3 pb-2 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-600">
				Módulos
			</p>
			<ul class="space-y-1">
				{#each MODULOS as modulo (modulo.ruta)}
					<li>
						<a
							href={modulo.ruta}
							aria-current={esActiva(modulo.ruta) ? 'page' : undefined}
							onclick={(e) => {
								e.preventDefault();
								goto(modulo.ruta);
							}}
							class="group flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition {esActiva(
								modulo.ruta
							)
								? 'bg-white/10 text-white shadow-sm ring-1 ring-white/10'
								: 'text-slate-400 hover:bg-white/5 hover:text-slate-100'}"
						>
							<span
								class="grid h-8 w-8 shrink-0 place-items-center rounded-lg transition {esActiva(
									modulo.ruta
								)
									? 'bg-indigo-500 text-white shadow-sm shadow-indigo-950/40'
									: 'bg-white/5 text-slate-400 group-hover:text-slate-200'}"
							>
								<IconoModulo ruta={modulo.ruta} clase="h-4 w-4" />
							</span>
							<span class="min-w-0 flex-1 truncate">{modulo.nombre}</span>
							<span class="kbd">{modulo.tecla}</span>
						</a>
					</li>
				{/each}
			</ul>
		</nav>

		<div class="border-t border-white/5 p-4">
			<div class="rounded-xl border border-white/5 bg-white/[0.03] p-3">
				<p class="text-[10px] font-bold uppercase tracking-[0.18em] text-slate-500">Estado</p>
				<div
					class="mt-2 flex items-center gap-2 text-xs font-semibold {appState.conectado
						? 'text-emerald-300'
						: 'text-rose-300'}"
				>
					<span
						class="h-2 w-2 rounded-full bg-current {appState.conectado
							? 'animate-pulse-soft'
							: ''}"
					></span>
					{appState.sincronizando
						? 'Sincronizando MP…'
						: appState.conectado
							? 'Operativo'
							: 'Sin conexión'}
				</div>
				<p class="mt-1 text-[11px] text-slate-500">Datos guardados localmente</p>
				<p class="mt-2 flex items-center gap-2 text-[11px] text-slate-500">
					<span class="h-1.5 w-1.5 rounded-full bg-sky-400"></span>
					Lector USB · HID
				</p>
			</div>
		</div>
	</aside>

	<div class="flex min-w-0 flex-1 flex-col">
		<header
			class="flex h-16 shrink-0 items-center justify-between gap-4 border-b border-slate-200/80 bg-white/80 px-5 backdrop-blur lg:px-8"
		>
			<div class="min-w-0">
				<p class="text-[10px] font-bold uppercase tracking-[0.2em] text-indigo-500">
					Kiosquitodo
				</p>
				<h1 class="truncate text-base font-bold tracking-tight text-slate-900">
					{moduloActual.nombre}
				</h1>
			</div>

			<div class="flex shrink-0 items-center gap-3">
				<span class="hidden text-sm capitalize text-slate-500 lg:block">{fechaLarga}</span>
				<span
					class="rounded-lg bg-slate-100 px-3 py-1.5 font-mono text-sm font-semibold tabular-nums text-slate-700"
				>
					{hora.toLocaleTimeString('es-AR', { hour: '2-digit', minute: '2-digit' })}
				</span>
				<span
					class="hidden items-center gap-2 rounded-full px-3 py-1.5 text-xs font-semibold md:inline-flex {appState.conectado
						? 'bg-emerald-50 text-emerald-700'
						: 'bg-rose-50 text-rose-700'}"
				>
					<span class="h-1.5 w-1.5 rounded-full bg-current"></span>
					{appState.sincronizando
						? 'Sincronizando'
						: appState.conectado
							? 'Conectado'
							: 'Sin conexión'}
				</span>
			</div>
		</header>

		<main class="min-h-0 flex-1 overflow-auto">
			{@render children()}
		</main>
	</div>
</div>

<Actualizacion />

{#if appState.bloqueado}
	<Bloqueo />
{/if}

{#if productoConsulta}
	<div
		class="pointer-events-none fixed inset-x-0 bottom-8 z-40 flex justify-center px-4"
		role="status"
		aria-live="polite"
	>
		<div
			class="pointer-events-auto w-full max-w-sm rounded-2xl border border-white/70 bg-white p-5 shadow-float animate-fade-up"
		>
			<div class="flex items-start justify-between gap-3">
				<div class="min-w-0">
					<p class="eyebrow">Consulta de precio</p>
					<p class="mt-1 truncate text-base font-bold text-slate-900">
						{productoConsulta.nombre}
					</p>
				</div>
				<button class="btn-ghost btn-sm" onclick={cerrarConsulta} aria-label="Cerrar consulta">
					✕
				</button>
			</div>
			<div class="mt-3 flex items-end justify-between gap-3">
				<p class="text-2xl font-black tabular-nums tracking-tight text-slate-900">
					{formatearPesos(productoConsulta.precio_venta)}
				</p>
				<p
					class="text-xs font-semibold {productoConsulta.stock > 0
						? 'text-emerald-600'
						: 'text-rose-600'}"
				>
					{productoConsulta.stock > 0 ? `Stock: ${productoConsulta.stock}` : 'Sin stock'}
				</p>
			</div>
		</div>
	</div>
{/if}

<Toast />