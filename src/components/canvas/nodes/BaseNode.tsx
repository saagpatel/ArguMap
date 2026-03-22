import type { Node } from "@xyflow/react";
import { Handle, type NodeProps, NodeResizer, Position } from "@xyflow/react";
import { type ReactNode, useEffect, useRef } from "react";
import {
	type ArgNodeData,
	NODE_CONFIG,
	NODE_DEFAULTS,
	STRENGTH_COLORS,
} from "../../../types";

interface BaseNodeProps extends NodeProps<Node<ArgNodeData>> {
	children?: ReactNode;
}

export default function BaseNode({
	id,
	data,
	selected,
	children,
}: BaseNodeProps) {
	const config = NODE_CONFIG[data.node_type];
	const textareaRef = useRef<HTMLTextAreaElement>(null);

	// Auto-focus new empty nodes
	useEffect(() => {
		if (selected && data.content === "" && textareaRef.current) {
			textareaRef.current.focus();
		}
	}, [selected, data.content]);

	// Auto-resize textarea
	const handleInput = () => {
		const el = textareaRef.current;
		if (el) {
			el.style.height = "auto";
			el.style.height = `${el.scrollHeight}px`;
		}
	};

	return (
		<div
			className="relative rounded-lg border-2 p-3 pt-7"
			style={
				{
					borderColor: config.border,
					backgroundColor: config.bg,
					"--handle-color": config.border,
				} as React.CSSProperties
			}
		>
			<NodeResizer
				color={config.border}
				isVisible={selected ?? false}
				minWidth={NODE_DEFAULTS.minWidth}
				minHeight={NODE_DEFAULTS.minHeight}
			/>

			{/* Type badge */}
			<span
				className="absolute left-2 top-1 rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-white"
				style={{ backgroundColor: config.border }}
			>
				{config.label}
			</span>

			{/* Collapse toggle — only shown if node has descendants */}
			{data.onToggleCollapse && (
				<button
					onClick={(e) => {
						e.stopPropagation();
						data.onToggleCollapse?.(id);
					}}
					className="nodrag nopan absolute right-2 top-1 flex items-center gap-1 rounded px-1 py-0.5 text-[10px] text-zinc-400 transition-colors hover:bg-white/10 hover:text-zinc-200"
				>
					<svg
						className="h-3 w-3 transition-transform"
						style={{
							transform: data.isCollapsed ? "rotate(-90deg)" : "rotate(0deg)",
						}}
						fill="none"
						stroke="currentColor"
						strokeWidth={2}
						viewBox="0 0 24 24"
					>
						<path
							strokeLinecap="round"
							strokeLinejoin="round"
							d="M19 9l-7 7-7-7"
						/>
					</svg>
					{data.isCollapsed && data.hiddenDescendantCount
						? data.hiddenDescendantCount
						: null}
				</button>
			)}

			{/* Connection handles — 4 directions */}
			<Handle
				type="target"
				position={Position.Top}
				id="top"
				className="!h-2.5 !w-2.5 !rounded-full !border-2"
				style={{ borderColor: config.border, backgroundColor: config.bg }}
			/>
			<Handle
				type="source"
				position={Position.Bottom}
				id="bottom"
				className="!h-2.5 !w-2.5 !rounded-full !border-2"
				style={{ borderColor: config.border, backgroundColor: config.bg }}
			/>
			<Handle
				type="target"
				position={Position.Left}
				id="left"
				className="!h-2.5 !w-2.5 !rounded-full !border-2"
				style={{ borderColor: config.border, backgroundColor: config.bg }}
			/>
			<Handle
				type="source"
				position={Position.Right}
				id="right"
				className="!h-2.5 !w-2.5 !rounded-full !border-2"
				style={{ borderColor: config.border, backgroundColor: config.bg }}
			/>

			{/* Content textarea */}
			<textarea
				ref={textareaRef}
				className="nodrag nowheel nopan w-full resize-none bg-transparent text-sm text-zinc-100 placeholder:text-zinc-500 outline-none"
				placeholder="Enter content..."
				value={data.content}
				onChange={(e) => data.onUpdate(id, { content: e.target.value })}
				onInput={handleInput}
				rows={1}
			/>

			{/* Strength indicator bar */}
			{data.strength != null && (
				<div
					className="mt-1 h-1 rounded-full"
					style={{
						backgroundColor: STRENGTH_COLORS[data.strength],
						width: `${(data.strength / 5) * 100}%`,
					}}
				/>
			)}

			{/* Extension slot for EvidenceNode source input */}
			{children}
		</div>
	);
}
