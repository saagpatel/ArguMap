import { useReactFlow } from "@xyflow/react";
import { useCallback, useEffect, useRef } from "react";
import { tauriApi } from "../lib/tauri";
import type {
	ArgFlowEdge,
	ArgFlowNode,
	EdgePayload,
	NodePayload,
} from "../types";

export function useMapSync(mapId: string | null) {
	const isHydrating = useRef(false);
	const positionDebounceRef = useRef<ReturnType<typeof setTimeout>>();
	const contentDebounceRef = useRef<ReturnType<typeof setTimeout>>();
	const { getNodes, getEdges } = useReactFlow<ArgFlowNode, ArgFlowEdge>();

	const flush = useCallback(() => {
		if (!mapId || isHydrating.current) return;

		const nodes: NodePayload[] = getNodes().map((n) => ({
			id: n.id,
			node_type: n.data.node_type,
			content: n.data.content,
			source: n.data.source,
			x: n.position.x,
			y: n.position.y,
			width: n.measured?.width ?? n.data.width ?? 220,
			height: n.measured?.height ?? n.data.height ?? 80,
		}));

		const edges: EdgePayload[] = getEdges().map((e) => ({
			id: e.id,
			source_node_id: e.source,
			target_node_id: e.target,
			edge_type: e.data?.edge_type ?? "supports",
			label: e.data?.label,
		}));

		tauriApi.saveMapState(mapId, nodes, edges).catch(console.error);
	}, [mapId, getNodes, getEdges]);

	const syncOnPosition = useCallback(() => {
		clearTimeout(positionDebounceRef.current);
		positionDebounceRef.current = setTimeout(flush, 500);
	}, [flush]);

	const syncOnContent = useCallback(() => {
		clearTimeout(contentDebounceRef.current);
		contentDebounceRef.current = setTimeout(flush, 1000);
	}, [flush]);

	const syncImmediate = useCallback(() => {
		flush();
	}, [flush]);

	// Cleanup timers on unmount
	useEffect(() => {
		return () => {
			clearTimeout(positionDebounceRef.current);
			clearTimeout(contentDebounceRef.current);
		};
	}, []);

	return { isHydrating, syncOnPosition, syncOnContent, syncImmediate };
}
