<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { rust } from '@codemirror/lang-rust';
	import { EditorView } from '@codemirror/view';
	import { indentUnit } from '@codemirror/language';
	import { Prec } from '@codemirror/state';

	let { src = $bindable(), dotColor = $bindable() } = $props<{
		src: string;
		dotColor: string;
	}>();

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

	$effect(() => {
		src;
		dotColor = '#f59e0b';
	});
</script>

<CodeMirror
	bind:value={src}
	lang={rust()}
	theme={minimalTheme}
	extensions={[Prec.highest(indentUnit.of('    '))]}
/>
