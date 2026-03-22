import { useEffect, useRef } from "react";
import {
	type ArgFlowNode,
	type ArgNodeData,
	NODE_CONFIG,
	type NodeType,
	STRENGTH_COLORS,
} from "../../types";

const NODE_TYPE_OPTIONS: { value: NodeType; label: string }[] = [
	{ value: "claim", label: "Claim" },
	{ value: "evidence", label: "Evidence" },
	{ value: "rebuttal", label: "Rebuttal" },
	{ value: "counter_rebuttal", label: "Counter-Rebuttal" },
];

interface NodeEditorProps {
	node: ArgFlowNode;
	onUpdate: ArgNodeData["onUpdate"];
}

export default function NodeEditor({ node, onUpdate }: NodeEditorProps) {
	const textareaRef = useRef<HTMLTextAreaElement>(null);

	// Auto-resize textarea on content change and mount
	useEffect(() => {
		const el = textareaRef.current;
		if (el) {
			el.style.height = "auto";
			el.style.height = `${el.scrollHeight}px`;
		}
	}, [node.data.content, node.id]);

	return (
		<div className="flex flex-col gap-3">
			<h3 className="text-xs font-semibold uppercase tracking-wider text-zinc-500">
				Edit Node
			</h3>

			{/* Type selector */}
			<div>
				<label className="mb-1 block text-xs text-zinc-400">Type</label>
				<select
					value={node.data.node_type}
					onChange={(e) =>
						onUpdate(node.id, { node_type: e.target.value as NodeType })
					}
					className="w-full"
				>
					{NODE_TYPE_OPTIONS.map(({ value, label }) => (
						<option key={value} value={value}>
							{label}
						</option>
					))}
				</select>
			</div>

			{/* Node type color indicator */}
			<div
				className="h-1 w-full rounded-full"
				style={{ backgroundColor: NODE_CONFIG[node.data.node_type].border }}
			/>

			{/* Content textarea */}
			<div>
				<label className="mb-1 block text-xs text-zinc-400">Content</label>
				<textarea
					ref={textareaRef}
					value={node.data.content}
					onChange={(e) => {
						onUpdate(node.id, { content: e.target.value });
						const el = e.target;
						el.style.height = "auto";
						el.style.height = `${el.scrollHeight}px`;
					}}
					className="w-full resize-none rounded-md border border-zinc-700 bg-[#1C1C1C] p-2 text-sm text-zinc-100 outline-none focus:border-zinc-500"
					rows={3}
					placeholder="Enter content..."
				/>
			</div>

			{/* Source field — only for evidence nodes */}
			{node.data.node_type === "evidence" && (
				<div>
					<label className="mb-1 block text-xs text-zinc-400">Source</label>
					<input
						value={node.data.source ?? ""}
						onChange={(e) => onUpdate(node.id, { source: e.target.value })}
						className="w-full rounded-md border border-zinc-700 bg-[#1C1C1C] p-2 text-sm text-zinc-100 outline-none focus:border-zinc-500"
						placeholder="URL or citation"
					/>
				</div>
			)}

			{/* Strength scoring */}
			<div>
				<label className="mb-1 block text-xs text-zinc-400">Strength</label>
				<div className="flex items-center gap-1.5">
					{[1, 2, 3, 4, 5].map((level) => (
						<button
							key={level}
							onClick={() =>
								onUpdate(node.id, {
									strength: node.data.strength === level ? undefined : level,
								})
							}
							className="h-6 w-6 rounded-full border-2 transition-all hover:scale-110"
							style={{
								borderColor: STRENGTH_COLORS[level],
								backgroundColor:
									node.data.strength !== undefined &&
									node.data.strength >= level
										? STRENGTH_COLORS[level]
										: "transparent",
							}}
							title={`Strength ${level}`}
						/>
					))}
					<span className="ml-2 text-xs text-zinc-500">
						{node.data.strength ? `${node.data.strength}/5` : "Not rated"}
					</span>
				</div>
			</div>
		</div>
	);
}
