import { invoke } from '@tauri-apps/api/core';

export async function llamar<T>(comando: string, args?: Record<string, unknown>): Promise<T> {
	return invoke<T>(comando, args);
}