<script lang="ts">
	import { Spinner } from 'flowbite-svelte';
	import { Play } from 'lucide-svelte';
	import { CloudDownload, Info, Copy, Share2 } from 'lucide-svelte';
	import { DropdownItem, Dropdown, DropdownDivider } from 'flowbite-svelte';
	import { browser } from '$app/environment';
	import Editor from '$lib/components/Editor.svelte';
	import Output from '$lib/components/Output.svelte';
	import type { RunResponse, RunResult, ExecutionStatus } from '$lib/types';

	let src = $state('fn main() {\n' + '    println!("Hello ft-lgtm!");\n' + '}');
	let compileStatus = $state(0);
	let ipfsStatus = $state(true);
	let responseStatus = $state(true);
	let stdout = $state('');
	let stderr = $state('');
	let executionError = $state<string | null>(null);

	let importCid = $state('');
	let leftWidth = $state(60);
	let container: HTMLDivElement;
	let dragging = $state(false);

	let dotColor = $state('#22c55e');
	let isRunning = $state(false);

	let shareOpen = $state(false);

	function sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	async function runCode() {
		isRunning = true;

		try {
			const response = await fetch('/api/execute', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					language: 'rust',
					src
				})
			});

			if (!response.ok) {
				throw new Error('Request failed');
			}

			responseStatus = true;

			const result: RunResponse = await response.json();

			handleRunResult(result);
		} catch (err) {
			responseStatus = false;
			console.error(err);
		} finally {
			isRunning = false;
		}
	}

	function handleRunResult(result: RunResponse) {
		const { run_result, cid } = result;
		const { compile_result, execution_result } = run_result;

		compileStatus = compile_result.status;
		executionError = executionErrorMessage(execution_result?.status);

		if (compileStatus !== 0) {
			dotColor = '#ef4444';
			stdout = compile_result.stdout;
			stderr = compile_result.stderr;
		} else {
			dotColor = '#22c55e';
			stdout = execution_result?.stdout ?? '';
			stderr = execution_result?.stderr ?? '';
		}

		if (cid) {
			ipfsStatus = true;
			addRun(cid);
		} else {
			ipfsStatus = false;
		}
	}

	function executionErrorMessage(status: ExecutionStatus | undefined): string | null {
		if (!status || 'Ok' in status) return null;
		return 'Exit' in status.Err ? `Program exited with code ${status.Err.Exit}` : status.Err.Trap;
	}

	async function fetchImportedCid() {
		if (!importCid.trim()) return;

		try {
			const [srcResponse, resultResponse] = await Promise.all([
				fetch(`http://ipfs.lgtm.local/ipfs/${importCid}/main.rs`),
				fetch(`http://ipfs.lgtm.local/ipfs/${importCid}/run_result.json`)
			]);

			if (!srcResponse.ok) {
				throw new Error(`Failed to fetch src: ${srcResponse.status}`);
			}
			if (!resultResponse.ok) {
				throw new Error(`Failed to fetch run_result.json: ${resultResponse.status}`);
			}

			const fetchedSrc = await srcResponse.text();
			const fetchedRunResult: RunResult = await resultResponse.json();

			src = fetchedSrc;
			stdout = fetchedRunResult.execution_result?.stdout ?? '';
			stderr = fetchedRunResult.execution_result?.stderr ?? '';
			compileStatus = fetchedRunResult.compile_result?.status ?? 0;
			executionError = executionErrorMessage(fetchedRunResult.execution_result?.status);
			shareOpen = false;
		} catch (err) {
			console.error('failed to fetch imported CID:', err);
		}
	}

	function startDrag(e: PointerEvent) {
		e.preventDefault();
		dragging = true;
		window.addEventListener('pointermove', onDrag);
		window.addEventListener('pointerup', stopDrag);
	}

	let shareWrapper: HTMLDivElement;

	function handleClickOutside(e: MouseEvent) {
		if (shareOpen && shareWrapper && !shareWrapper.contains(e.target as Node)) {
			shareOpen = false;
		}
	}

	function onDrag(e: PointerEvent) {
		if (!dragging || !container) return;
		const rect = container.getBoundingClientRect();
		const pct = ((e.clientX - rect.left) / rect.width) * 100;
		leftWidth = Math.min(80, Math.max(20, pct));
	}

	function stopDrag() {
		dragging = false;
		window.removeEventListener('pointermove', onDrag);
		window.removeEventListener('pointerup', stopDrag);
	}

	async function copyToClipboard(value: string) {
		try {
			await navigator.clipboard.writeText(value);
			console.log('copied:', value);
		} catch (err) {
			console.error('copy failed:', err);
		}
	}

	type Run = {
		cid: string;
		time: number;
	};

	let runList: Run[] = $state(browser ? loadRunList() : []);

	function addRun(cid: string) {
		runList = [
			...runList,
			{
				cid,
				time: Date.now()
			}
		];

		if (browser) {
			localStorage.setItem('runList', JSON.stringify(runList));
		}
	}

	function loadRunList(): Run[] {
		if (!browser) return [];
		try {
			const raw = localStorage.getItem('runList');
			return raw ? JSON.parse(raw) : [];
		} catch {
			return [];
		}
	}

	function formatTime(timestamp: number): string {
		const diff = Date.now() - timestamp;
		const seconds = Math.floor(diff / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);
		const days = Math.floor(hours / 24);

		if (seconds < 60) return 'just now';
		if (minutes < 60) return `${minutes}m ago`;
		if (hours < 24) return `${hours}h ago`;
		if (days < 7) return `${days}d ago`;

		return new Date(timestamp).toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric'
		});
	}
</script>

<svelte:window onclick={handleClickOutside} />

<div class="playground">
	<div class="titlebar">
		<div class="file">
			<span class="dot" style="background: {dotColor}"></span>
			<span class="filename">main.rs</span>
		</div>
		<div class="menu">
			<div class="share-wrapper" bind:this={shareWrapper}>
				<button class="share" onclick={() => (shareOpen = !shareOpen)}>
					<Share2 size="14" />
					Share
				</button>

				{#if shareOpen}
					<div class="dropdown">
						<div class="import-item">
							<span class="import-text"> Import </span>
							<input
								type="text"
								class="cid-input"
								placeholder="Paste a CID…"
								bind:value={importCid}
								onkeydown={(e) => e.key === 'Enter' && fetchImportedCid()}
							/>
							<button class="fetch-btn" onclick={fetchImportedCid} aria-label="Fetch CID">
								<CloudDownload size="14" />
							</button>
						</div>
						<div class="dropdown-divider"></div>
						{#each runList.slice().reverse().slice(0, 5) as entry}
							<div class="dropdown-item">
								<span class="dropdown-item-time">{formatTime(entry.time)}</span>
								<span class="dropdown-item-cid">{entry.cid}</span>
								<button class="copy-btn" onclick={() => copyToClipboard(entry.cid)}>
									<Copy size="14" />
								</button>
							</div>
						{/each}
					</div>
				{/if}
			</div>
			<button class="run" onclick={runCode} disabled={isRunning}>
				{#if isRunning}
					<Spinner type="dots" color="green" size={'12'} />
				{:else}
					<Play size="14" />
					Run
				{/if}
			</button>
		</div>
	</div>
	<div class="panes" bind:this={container} class:dragging>
		<div class="editor-pane" style="width: {leftWidth}%">
			<Editor bind:src bind:dotColor />
		</div>
		<div
			class="divider"
			role="separator"
			aria-orientation="vertical"
			onpointerdown={startDrag}
		></div>
		<div class="output-pane" style="width: {100 - leftWidth}%">
			<Output {responseStatus} {compileStatus} {ipfsStatus} {executionError} {stdout} {stderr} />
		</div>
	</div>
</div>

<style>
	.playground {
		border: 0.5px solid #ddd;
		border-radius: 12px;
		overflow: visible;
		width: 90%;
		max-width: 1000px;
		margin: 2rem auto;
		font-family: system-ui, sans-serif;
	}
	.share-wrapper {
		position: relative;
	}
	.dropdown {
		position: fixed;
		top: 80px;
		left: 50%;
		transform: translateX(-50%);
		background: #f5eee7;
		border: 0.5px solid #ddd;
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		max-width: 90vw;
		min-width: 160px;
		padding: 4px;
		z-index: 100;
	}
	.dropdown-divider {
		height: 1px;
		background: #e5ddd3;
		margin: 4px 0;
	}
	.dropdown-item-time {
		font-size: 9px;
		color: #999;
		flex-shrink: 0;
		white-space: nowrap;
	}
	.dropdown-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 10px;
		border-radius: 6px;
	}

	.dropdown-item:hover {
		background: #f5eee7;
	}

	.dropdown-item-cid {
		font-size: 11px;
		color: #6b7280;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.copy-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: none;
		cursor: pointer;
		color: #888;
		padding: 2px;
		flex-shrink: 0;
	}

	.copy-btn:hover {
		color: #333;
	}
	.titlebar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		border-bottom: 0.5px solid #ddd;
	}
	.menu {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.share {
		font-size: 13px;
		padding: 6px 14px;
		display: flex;
		align-items: center;
		gap: 6px;
		background: transparent;
		border: 0.5px solid transparent;
		border-radius: 6px;
		cursor: pointer;
		height: 28px;
		min-width: 76px;
		color: #555;
	}

	.share:hover {
		background: #f5eee7;
		border-color: #ccc;
	}
	.file {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
	}
	.filename {
		font-size: 13px;
		color: #555;
	}
	.run {
		font-size: 13px;
		padding: 6px 14px;
		display: flex;
		align-items: center;
		gap: 6px;
		background: #f5eee7;
		border: 0.5px solid #ccc;
		border-radius: 6px;
		cursor: pointer;
		height: 28px;
		min-width: 76px;
		justify-content: center;
	}
	.run:hover {
		filter: grayscale(10%) brightness(0.95);
	}
	.panes {
		display: flex;
		align-items: stretch;
	}
	.panes.dragging {
		user-select: none;
		cursor: col-resize;
	}
	.editor-pane {
		padding: 20px 24px;
		flex-shrink: 0;
		min-width: 0;
		box-sizing: border-box;
	}

	.divider {
		width: 5px;
		flex-shrink: 0;
		cursor: col-resize;
		background: transparent;
		position: relative;
	}
	.divider::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 0;
		bottom: 0;
		width: 1px;
		background: #ddd;
	}
	.divider:hover::after,
	.dragging .divider::after {
		background: #999;
	}
	.output-pane {
		padding: 20px 24px;
		flex-shrink: 0;
		min-width: 0;
		box-sizing: border-box;
	}
	.import-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
	}
	.import-text {
		display: flex;
		font-size: 13px;
		align-items: center;
		color: #999;
	}

	.cid-input {
		flex: 1;
		min-width: 0;
		font-size: 11px;
		padding: 6px 8px;
		border: 0.5px solid #ddd;
		border-radius: 6px;
		background: #f7f7f6;
		color: #999;
	}

	.cid-input:focus {
		outline: none;
		border-color: #999;
	}

	.fetch-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: none;
		cursor: pointer;
		color: #888;
		padding: 4px;
		flex-shrink: 0;
	}

	.fetch-btn:hover {
		color: #333;
	}
	@media (max-width: 600px) {
		.panes {
			flex-direction: column;
		}
		.editor-pane,
		.output-pane {
			width: 100% !important;
		}
		.divider {
			display: none;
		}
	}
</style>
