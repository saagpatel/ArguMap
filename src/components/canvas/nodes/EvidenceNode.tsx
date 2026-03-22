import type { Node, NodeProps } from "@xyflow/react";
import type { ArgNodeData } from "../../../types";
import BaseNode from "./BaseNode";

export default function EvidenceNode(props: NodeProps<Node<ArgNodeData>>) {
	return (
		<BaseNode {...props}>
			<input
				className="nodrag nowheel nopan mt-1 w-full border-t border-zinc-700 bg-transparent pt-1 text-xs text-zinc-400 placeholder:text-zinc-600 outline-none"
				placeholder="Source URL or citation"
				value={props.data.source ?? ""}
				onChange={(e) =>
					props.data.onUpdate(props.id, { source: e.target.value })
				}
			/>
		</BaseNode>
	);
}
