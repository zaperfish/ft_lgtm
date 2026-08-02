<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { rust } from '@codemirror/lang-rust';
	import { EditorView } from '@codemirror/view';
	import { indentUnit } from '@codemirror/language';
	import { Prec } from '@codemirror/state';
	import { Spinner } from 'flowbite-svelte';
	import { Play } from 'lucide-svelte';
	import { AlertCircle } from 'lucide-svelte';
	import { Copy, Share2 } from 'lucide-svelte';
	import { DropdownItem, Dropdown, DropdownDivider } from 'flowbite-svelte';
	import { browser } from '$app/environment';

	let src = $state('fn main() {\n' + '    println!("Hello ft-lgtm!");\n' + '}');
	let compileStatus = $state(0);
	let ipfsStatus = $state(true);
	let responseStatus = $state(true);
	let stdout = $state('');
	let stderr = $state('');

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
			const response = await fetch('/api/code/run', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					language: 'rust',
					src: src
				})
			});

			if (!response.ok) {
				responseStatus = false;
				throw new Error('Request failed');
			}
			responseStatus = true;

			const result = await response.json();
			console.log(result.run_result);
			compileStatus = result.run_result?.compile_result?.status;
			if (compileStatus != 0) {
				dotColor = '#ef4444';
				stdout = result.run_result?.compile_result?.stdout ?? '';
				stderr = result.run_result?.compile_result?.stderr ?? '';
			} else {
				dotColor = '#22c55e';
				stdout = result.run_result?.execution_result?.stdout ?? '';
				stderr = result.run_result?.execution_result?.stderr ?? '';
				console.log(result.run_result?.cid);
				if (result.run_result?.cid) {
					ipfsStatus = true;
					addRun(result.run_result?.cid);
				} else {
					ipfsStatus = false;
				}
			}
			console.log(result.output);
		} catch (err) {
			console.error(err);
		} finally {
			isRunning = false;
		}
	}

	$effect(() => {
		src;
		dotColor = '#f59e0b';
	});

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

	const minimalTheme = EditorView.theme({
		'&': {
			fontSize: '13px',
			backgroundColor: 'transparent'
		},
		'.cm-content': {
			fontFamily: "ui-monospace, 'SF Mono', monospace",
			padding: '0'
		},
		'.cm-gutters': {
			backgroundColor: 'transparent',
			border: 'none',
			color: '#bbb'
		},
		'.cm-activeLine': {
			backgroundColor: 'transparent'
		},
		'.cm-activeLineGutter': {
			backgroundColor: 'transparent'
		},
		'&.cm-focused': {
			outline: 'none'
		}
	});
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
			<CodeMirror
				bind:value={src}
				lang={rust()}
				theme={minimalTheme}
				extensions={[Prec.highest(indentUnit.of('    '))]}
			/>
		</div>
		<div
			class="divider"
			role="separator"
			aria-orientation="vertical"
			onpointerdown={startDrag}
		></div>
		<div class="output-pane" style="width: {100 - leftWidth}%">
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
				<p class="output-label1">Stdout</p>
				<p class="output-content">{stdout}</p>
				<p class="output-label2">Stderr</p>
				<p class="output-content">{stderr}</p>
			{/if}
		</div>
	</div>
</div>

<style>
	.playground {
		border: 0.5px solid #ddd;
		border-radius: 12px;
		overflow: visible;
		max-width: 1000px;
		margin: 2rem auto;
		font-family: system-ui, sans-serif;
	}
	.share-wrapper {
		position: relative;
	}
	.dropdown {
		position: absolute;
		top: calc(100% + 6px);
		left: 50%;
		transform: translateX(-50%);
		background: #f5eee7;
		border: 0.5px solid #ddd;
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		min-width: 160px;
		padding: 4px;
		z-index: 100;
	}
	.dropdown-item-time {
		font-size: 11px;
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
		font-size: 13px;
		color: #333;
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
		box-sizing: border-box;
	}
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
