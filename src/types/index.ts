import type { Edge, Node } from "@xyflow/react";

export type NodeType = "claim" | "evidence" | "rebuttal" | "counter_rebuttal";
export type EdgeType = "supports" | "rebuts" | "qualifies" | "depends_on";

export interface ArgMap {
	id: string;
	title: string;
	description?: string;
	created_at: string;
	updated_at: string;
}

export interface ArgNode {
	id: string;
	map_id: string;
	node_type: NodeType;
	content: string;
	source?: string;
	x: number;
	y: number;
	width: number;
	height: number;
	strength?: number;
}

export interface ArgEdge {
	id: string;
	map_id: string;
	source_node_id: string;
	target_node_id: string;
	edge_type: EdgeType;
	label?: string;
}

export interface NodePayload {
	id: string;
	node_type: NodeType;
	content: string;
	source?: string;
	x: number;
	y: number;
	width: number;
	height: number;
	strength?: number;
}

export interface EdgePayload {
	id: string;
	source_node_id: string;
	target_node_id: string;
	edge_type: EdgeType;
	label?: string;
}

export type ArgNodeData = {
	node_type: NodeType;
	content: string;
	source?: string;
	width?: number;
	height?: number;
	strength?: number;
	// Ephemeral collapse UI state — not synced to DB
	isCollapsed?: boolean;
	hiddenDescendantCount?: number;
	onToggleCollapse?: (id: string) => void;
	onUpdate: (
		id: string,
		updates: Partial<
			Omit<
				ArgNodeData,
				| "onUpdate"
				| "onToggleCollapse"
				| "isCollapsed"
				| "hiddenDescendantCount"
			>
		>,
	) => void;
};

export const STRENGTH_COLORS: Record<number, string> = {
	1: "#EF4444", // red — very weak
	2: "#F97316", // orange — weak
	3: "#FBBF24", // amber — moderate
	4: "#84CC16", // lime — strong
	5: "#10B981", // green — very strong
};

export const EDGE_COLORS: Record<EdgeType, string> = {
	supports: "#10B981",
	rebuts: "#EF4444",
	qualifies: "#FBBF24",
	depends_on: "#6B7280",
};

export const NODE_CONFIG: Record<
	NodeType,
	{ border: string; bg: string; label: string }
> = {
	claim: { border: "#3B82F6", bg: "#1E3A5F", label: "Claim" },
	evidence: { border: "#10B981", bg: "#0F3028", label: "Evidence" },
	rebuttal: { border: "#EF4444", bg: "#3B1212", label: "Rebuttal" },
	counter_rebuttal: {
		border: "#F97316",
		bg: "#3B1E0A",
		label: "Counter-Rebuttal",
	},
};

export const NODE_DEFAULTS = {
	width: 220,
	height: 80,
	minWidth: 160,
	minHeight: 60,
} as const;

export type ArgFlowNode = Node<ArgNodeData, NodeType>;
export type ArgFlowEdge = Edge<{ edge_type: EdgeType; label?: string }>;
