<script lang="ts">
	import { appState, cerrar, type Notificacion, type TipoNotificacion } from '$lib/stores/app.svelte';

	const estilos: Record<TipoNotificacion, string> = {
		info: 'bg-sky-600',
		success: 'bg-emerald-600',
		warn: 'bg-amber-500',
		error: 'bg-rose-600'
	};

	const principales = $derived(appState.toasts.filter((t) => t.zona === 'abajo-derecha'));
	const secundarias = $derived(appState.toasts.filter((t) => t.zona === 'arriba-izquierda'));
</script>

{#snippet tarjetas(lista: Notificacion[])}
	{#each lista as toast (toast.id)}
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
{/snippet}

{#if principales.length > 0}
	<div class="fixed bottom-5 right-5 z-50 flex w-80 flex-col gap-2" aria-live="polite">
		{@render tarjetas(principales)}
	</div>
{/if}

{#if secundarias.length > 0}
	<div
		class="fixed left-5 top-20 z-40 flex w-56 flex-col gap-2 sm:left-[17.5rem] sm:top-5"
		aria-live="polite"
	>
		{@render tarjetas(secundarias)}
	</div>
{/if}
