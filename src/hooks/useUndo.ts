import { type Dispatch, type SetStateAction, useCallback, useRef } from "react";
import type { ArgFlowEdge, ArgFlowNode } from "../types";

type UndoAction =
	| { type: "add_node"; nodeId: string }
	| { type: "delete_node"; node: ArgFlowNode; connectedEdges: ArgFlowEdge[] }
	| { type: "add_edge"; edgeId: string }
	| { type: "delete_edge"; edge: ArgFlowEdge };

export function useUndo(
	setNodes: Dispatch<SetStateAction<ArgFlowNode[]>>,
	setEdges: Dispatch<SetStateAction<ArgFlowEdge[]>>,
	syncImmediate: () => void,
) {
	const lastAction = useRef<UndoAction | null>(null);

	const clear = useCallback(() => {
		lastAction.current = null;
	}, []);

	const recordAddNode = useCallback((nodeId: string) => {
		lastAction.current = { type: "add_node", nodeId };
	}, []);

	const recordDeleteNodes = useCallback(
		(nodes: ArgFlowNode[], edges: ArgFlowEdge[]) => {
			if (nodes.length === 1) {
				lastAction.current = {
					type: "delete_node",
					node: nodes[0],
					connectedEdges: edges,
				};
			}
		},
		[],
	);

	const recordAddEdge = useCallback((edgeId: string) => {
		lastAction.current = { type: "add_edge", edgeId };
	}, []);

	const recordDeleteEdge = useCallback((edge: ArgFlowEdge) => {
		lastAction.current = { type: "delete_edge", edge };
	}, []);

	const undo = useCallback(() => {
		const action = lastAction.current;
		if (!action) return;

		switch (action.type) {
			case "add_node":
				setNodes((nds) => nds.filter((n) => n.id !== action.nodeId));
				break;
			case "delete_node":
				setNodes((nds) => [...nds, action.node]);
				setEdges((eds) => [...eds, ...action.connectedEdges]);
				break;
			case "add_edge":
				setEdges((eds) => eds.filter((e) => e.id !== action.edgeId));
				break;
			case "delete_edge":
				setEdges((eds) => [...eds, action.edge]);
				break;
		}

		lastAction.current = null;
		syncImmediate();
	}, [setNodes, setEdges, syncImmediate]);

	return {
		recordAddNode,
		recordDeleteNodes,
		recordAddEdge,
		recordDeleteEdge,
		undo,
		clear,
	};
}
