<script lang="ts">
	import { onMount } from 'svelte';
	import {
		actualizaciones,
		descartarActualizacion,
		iniciarActualizaciones,
		instalarActualizacionDisponible
	} from '$lib/stores/actualizaciones.svelte';
	import { appState } from '$lib/stores/app.svelte';

	onMount(() => {
		void iniciarActualizaciones();
	});

	const porcentaje = $derived(
		actualizaciones.total && actualizaciones.total > 0
			? Math.min(100, Math.round((actualizaciones.descargado / actualizaciones.total) * 100))
			: null
	);
</script>

{#if actualizaciones.modalVisible && actualizaciones.disponible && !appState.bloqueado}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/75 p-4 backdrop-blur-sm"
		role="dialog"
		aria-modal="true"
		aria-labelledby="titulo-actualizacion"
	>
		<div class="modal-panel max-w-md">
			<div class="flex items-start justify-between gap-3">
				<div>
					<p class="eyebrow">Actualización</p>
					<h2 id="titulo-actualizacion" class="modal-title mt-1">Nueva versión disponible</h2>
				</div>
				<span class="badge-info">v{actualizaciones.disponible.version_nueva}</span>
			</div>

			<p class="mt-3 text-sm text-slate-600">
				Tenés la versión <span class="font-semibold">{actualizaciones.disponible.version_actual}</span>.
				La actualización se descarga, se verifica con la firma digital y se instala sola.
			</p>

			{#if actualizaciones.disponible.notas}
				<div class="mt-3 rounded-xl border border-slate-200 bg-slate-50 px-4 py-3">
					<p class="text-[11px] font-bold uppercase tracking-wider text-slate-500">Novedades</p>
					<p class="mt-1 whitespace-pre-line text-sm text-slate-700">
						{actualizaciones.disponible.notas}
					</p>
				</div>
			{/if}

			{#if actualizaciones.instalando}
				<div class="mt-4">
					<div class="flex items-center justify-between text-xs font-semibold text-slate-600">
						<span>Descargando e instalando…</span>
						<span>{porcentaje !== null ? `${porcentaje}%` : '…'}</span>
					</div>
					<div class="mt-2 h-2 overflow-hidden rounded-full bg-slate-200">
						<div
							class="h-full rounded-full bg-indigo-600 transition-all"
							style="width: {porcentaje ?? 5}%"
						></div>
					</div>
					<p class="mt-2 text-[11px] text-slate-500">
						La app se va a reiniciar cuando termine. No cierres la ventana.
					</p>
				</div>
			{/if}

			{#if actualizaciones.error}
				<p class="alert-error mt-4" role="alert">{actualizaciones.error}</p>
			{/if}

			<div class="mt-5 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
				<button
					onclick={() => void instalarActualizacionDisponible()}
					disabled={actualizaciones.instalando}
					class="btn-primary"
				>
					{actualizaciones.instalando ? 'Instalando…' : 'Instalar y reiniciar'}
				</button>
				<button
					onclick={descartarActualizacion}
					disabled={actualizaciones.instalando}
					class="btn-secondary"
				>
					Más tarde
				</button>
			</div>
		</div>
	</div>
{/if}
