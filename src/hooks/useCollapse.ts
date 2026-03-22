import { useCallback, useMemo, useState } from "react";
import type { ArgFlowEdge } from "../types";

function getDescendants(nodeId: string, edges: ArgFlowEdge[]): Set<string> {
	const visited = new Set<string>();
	const queue = [nodeId];

	while (queue.length > 0) {
		const current = queue.shift()!;
		for (const edge of edges) {
			if (edge.target === current && !visited.has(edge.source)) {
				visited.add(edge.source);
				queue.push(edge.source);
			}
		}
	}

	return visited; // does NOT include nodeId itself
}

export function useCollapse(edges: ArgFlowEdge[]) {
	const [collapsedNodeIds, setCollapsedNodeIds] = useState<Set<string>>(
		new Set(),
	);

	const hiddenNodeIds = useMemo(() => {
		const hidden = new Set<string>();
		for (const nodeId of collapsedNodeIds) {
			const descendants = getDescendants(nodeId, edges);
			for (const d of descendants) {
				hidden.add(d);
			}
		}
		return hidden;
	}, [collapsedNodeIds, edges]);

	const toggleCollapse = useCallback((nodeId: string) => {
		setCollapsedNodeIds((prev) => {
			const next = new Set(prev);
			if (next.has(nodeId)) {
				next.delete(nodeId);
			} else {
				next.add(nodeId);
			}
			return next;
		});
	}, []);

	const getDescendantCount = useCallback(
		(nodeId: string) => getDescendants(nodeId, edges).size,
		[edges],
	);

	const hasDescendants = useCallback(
		(nodeId: string) => edges.some((e) => e.target === nodeId),
		[edges],
	);

	const clear = useCallback(() => {
		setCollapsedNodeIds(new Set());
	}, []);

	return {
		collapsedNodeIds,
		hiddenNodeIds,
		toggleCollapse,
		getDescendantCount,
		hasDescendants,
		clear,
	};
}
