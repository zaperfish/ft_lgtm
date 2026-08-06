export interface CompileResult {
	status: number;
	stdout: string;
	stderr: string;
}

export type ExecutionError = { Exit: number } | { Trap: string };

export type ExecutionStatus = { Ok: null } | { Err: ExecutionError };

export interface ExecutionResult {
	status: ExecutionStatus;
	stdout: string;
	stderr: string;
}

export interface RunResult {
	compile_result: CompileResult;
	execution_result: ExecutionResult | null;
}

export interface RunResponse {
	run_result: RunResult;
	cid: string | null;
}
