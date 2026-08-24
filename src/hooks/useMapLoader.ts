import { useReactFlow } from "@xyflow/react";
import { useEffect } from "react";
import { tauriApi } from "../lib/tauri";
import type { ArgFlowEdge, ArgFlowNode, ArgNodeData } from "../types";

export function useMapLoader(
	mapId: string | null,
	isHydrating: React.MutableRefObject<boolean>,
	onUpdate: ArgNodeData["onUpdate"],
	reloadToken = 0,
) {
	const { setNodes, setEdges } = useReactFlow<ArgFlowNode, ArgFlowEdge>();

	useEffect(() => {
		if (!mapId) return;

		let cancelled = false;
		isHydrating.current = true;

		tauriApi
			.loadMap(mapId)
			.then(({ nodes, edges }) => {
				if (cancelled) {
					isHydrating.current = false;
					return;
				}

				const flowNodes: ArgFlowNode[] = nodes.map((node) => ({
					id: node.id,
					type: node.node_type as ArgFlowNode["type"],
					position: { x: node.x, y: node.y },
					width: node.width,
					height: node.height,
					data: {
						node_type: node.node_type,
						content: node.content,
						source: node.source,
						width: node.width,
						height: node.height,
						strength: node.strength,
						onUpdate,
					},
				}));

				const flowEdges: ArgFlowEdge[] = edges.map((edge) => ({
					id: edge.id,
					source: edge.source_node_id,
					target: edge.target_node_id,
					data: {
						edge_type: edge.edge_type,
						label: edge.label,
					},
				}));

				setNodes(flowNodes);
				setEdges(flowEdges);

				// Zero-delay setTimeout ensures React Flow processes the state update
				// and fires onNodesChange BEFORE we re-enable sync writes
				setTimeout(() => {
					if (!cancelled) {
						isHydrating.current = false;
					}
				}, 0);
			})
			.catch((err: unknown) => {
				console.error("Failed to load map:", err);
				isHydrating.current = false;
			});

		return () => {
			cancelled = true;
		};
	}, [mapId, reloadToken]); // eslint-disable-line react-hooks/exhaustive-deps
	// onUpdate is stable (ref-based), setNodes/setEdges are stable from useReactFlow
}
