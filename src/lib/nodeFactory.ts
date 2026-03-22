import { v4 as uuidv4 } from "uuid";
import type { ArgFlowNode, ArgNodeData, NodeType } from "../types";
import { NODE_DEFAULTS } from "../types";

export function createNode(
	type: NodeType,
	position: { x: number; y: number },
	onUpdate: ArgNodeData["onUpdate"],
): ArgFlowNode {
	return {
		id: uuidv4(),
		type,
		position,
		width: NODE_DEFAULTS.width,
		height: NODE_DEFAULTS.height,
		selected: true,
		data: {
			node_type: type,
			content: "",
			source: type === "evidence" ? "" : undefined,
			width: NODE_DEFAULTS.width,
			height: NODE_DEFAULTS.height,
			onUpdate,
		},
	};
}
