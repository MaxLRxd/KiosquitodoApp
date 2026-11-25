<script lang="ts">
	import { appState, cerrar, type TipoNotificacion } from '$lib/stores/app.svelte';

	const estilos: Record<TipoNotificacion, string> = {
		info: 'bg-sky-600',
		success: 'bg-emerald-600',
		warn: 'bg-amber-500',
		error: 'bg-rose-600'
	};
</script>

{#if appState.toasts.length > 0}
	<div class="fixed bottom-5 right-5 z-50 flex w-80 flex-col gap-2" aria-live="polite">
		{#each appState.toasts as toast (toast.id)}
			<div
				class="flex animate-fade-up items-start justify-between gap-3 rounded-xl px-4 py-3 text-sm font-medium text-white shadow-float {estilos[
					toast.tipo
				]}"
				role="alert"
			>
				<span>{toast.mensaje}</span>
				<button
					onclick={() => cerrar(toast.id)}
					class="grid h-5 w-5 shrink-0 place-items-center rounded-md text-white/70 transition hover:bg-white/20 hover:text-white"
					aria-label="Cerrar"
				>
					×
				</button>
			</div>
		{/each}
	</div>
{/if}
