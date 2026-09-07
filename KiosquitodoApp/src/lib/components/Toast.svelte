<script lang="ts">
	import { appState, cerrar, type TipoNotificacion } from '$lib/stores/app.svelte';

	const estilos: Record<TipoNotificacion, string> = {
		info: 'bg-sky-600',
		success: 'bg-green-600',
		warn: 'bg-amber-500',
		error: 'bg-red-600'
	};
</script>

{#if appState.toasts.length > 0}
	<div class="fixed bottom-16 right-4 z-50 flex w-80 flex-col gap-2" aria-live="polite">
		{#each appState.toasts as toast (toast.id)}
			<div
				class="flex items-start justify-between rounded-lg px-4 py-3 text-sm text-white shadow-lg {estilos[
					toast.tipo
				]}"
				role="alert"
			>
				<span>{toast.mensaje}</span>
				<button onclick={() => cerrar(toast.id)} class="ml-3 font-bold" aria-label="Cerrar">×</button>
			</div>
		{/each}
	</div>
{/if}