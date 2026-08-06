<script lang="ts">
	import { AlertCircle } from 'lucide-svelte';

	let { responseStatus, compileStatus, ipfsStatus, executionError, stdout, stderr } = $props<{
		responseStatus: boolean;
		compileStatus: number;
		ipfsStatus: boolean;
		executionError: string | null;
		stdout: string;
		stderr: string;
	}>();
</script>

{#if responseStatus == false}
	<div class="compile-error">
		<AlertCircle size="15" />
		<span>Oh no something went wrong (≧︿≦)</span>
	</div>
	<p class="output-content">{stderr}</p>
{:else if compileStatus != 0}
	<div class="compile-error">
		<AlertCircle size="15" />
		<span>Failed to compile</span>
	</div>
	<p class="output-content">{stderr}</p>
{:else}
	{#if ipfsStatus == false}
		<div class="ipfs-error">
			<AlertCircle size="15" />
			<span>Failed to publish to ipfs</span>
		</div>
	{/if}
	{#if executionError}
		<div class="compile-error">
			<AlertCircle size="15" />
			<span>{executionError}</span>
		</div>
	{/if}
	<p class="output-label1">Stderr</p>
	<p class="output-content">{stderr}</p>
	<p class="output-label2">Stdout</p>
	<p class="output-content">{stdout}</p>
{/if}

<style>
	.compile-error {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 500;
		color: #a3312a;
		background: #fbe9e7;
		border-radius: 6px;
		padding: 8px 12px;
		margin: 0 0 12px;
	}
	.ipfs-error {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 500;
		color: #f59e0b;
		background: #fdf1da;
		border-radius: 6px;
		padding: 8px 12px;
		margin: 0 0 12px;
	}
	.output-label1 {
		font-size: 12px;
		color: #999;
		margin: 0px 0 10px;
	}
	.output-label2 {
		font-size: 12px;
		color: #999;
		margin: 16px 0 10px;
	}
	.output-content {
		font-family: ui-monospace, 'SF Mono', monospace;
		font-size: 13px;
		color: #555;
		margin: 0;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
</style>
