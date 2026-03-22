import { v4 as uuidv4 } from "uuid";
import type { EdgePayload, NodePayload } from "../types";

interface TemplateResult {
	nodes: NodePayload[];
	edges: EdgePayload[];
}

function node(
	id: string,
	nodeType: NodePayload["node_type"],
	content: string,
	x: number,
	y: number,
	source?: string,
): NodePayload {
	return {
		id,
		node_type: nodeType,
		content,
		source,
		x,
		y,
		width: 260,
		height: 80,
	};
}

function edge(
	id: string,
	sourceId: string,
	targetId: string,
	edgeType: EdgePayload["edge_type"],
): EdgePayload {
	return {
		id,
		source_node_id: sourceId,
		target_node_id: targetId,
		edge_type: edgeType,
	};
}

export function createFiveWhysTemplate(): TemplateResult {
	const ids = Array.from({ length: 6 }, () => uuidv4());
	const eids = Array.from({ length: 5 }, () => uuidv4());

	return {
		nodes: [
			node(ids[0], "claim", "Problem Statement", 300, 0),
			node(ids[1], "evidence", "Why did this happen? (1)", 300, 160),
			node(ids[2], "evidence", "Why? (2)", 300, 320),
			node(ids[3], "evidence", "Why? (3)", 300, 480),
			node(ids[4], "evidence", "Root cause? (4)", 300, 640),
			node(ids[5], "evidence", "Deepest root cause (5)", 300, 800),
		],
		edges: [
			edge(eids[0], ids[1], ids[0], "supports"),
			edge(eids[1], ids[2], ids[1], "depends_on"),
			edge(eids[2], ids[3], ids[2], "depends_on"),
			edge(eids[3], ids[4], ids[3], "depends_on"),
			edge(eids[4], ids[5], ids[4], "depends_on"),
		],
	};
}

export function createProConTemplate(): TemplateResult {
	const ids = Array.from({ length: 5 }, () => uuidv4());
	const eids = Array.from({ length: 4 }, () => uuidv4());

	return {
		nodes: [
			node(ids[0], "claim", "Decision or Position", 350, 0),
			node(ids[1], "evidence", "Pro 1: Supporting argument", 50, 250),
			node(ids[2], "evidence", "Pro 2: Supporting argument", 300, 250),
			node(ids[3], "rebuttal", "Con 1: Counter-argument", 550, 250),
			node(ids[4], "rebuttal", "Con 2: Counter-argument", 800, 250),
		],
		edges: [
			edge(eids[0], ids[1], ids[0], "supports"),
			edge(eids[1], ids[2], ids[0], "supports"),
			edge(eids[2], ids[3], ids[0], "rebuts"),
			edge(eids[3], ids[4], ids[0], "rebuts"),
		],
	};
}

export function createMeceTemplate(): TemplateResult {
	const ids = Array.from({ length: 7 }, () => uuidv4());
	const eids = Array.from({ length: 6 }, () => uuidv4());

	return {
		nodes: [
			node(ids[0], "claim", "Main Thesis", 350, 0),
			node(ids[1], "claim", "Bucket A", 50, 220),
			node(ids[2], "claim", "Bucket B", 350, 220),
			node(ids[3], "claim", "Bucket C", 650, 220),
			node(ids[4], "evidence", "Data supporting A", 50, 440),
			node(ids[5], "evidence", "Data supporting B", 350, 440),
			node(ids[6], "evidence", "Data supporting C", 650, 440),
		],
		edges: [
			edge(eids[0], ids[1], ids[0], "qualifies"),
			edge(eids[1], ids[2], ids[0], "qualifies"),
			edge(eids[2], ids[3], ids[0], "qualifies"),
			edge(eids[3], ids[4], ids[1], "supports"),
			edge(eids[4], ids[5], ids[2], "supports"),
			edge(eids[5], ids[6], ids[3], "supports"),
		],
	};
}

export const TEMPLATES = {
	five_whys: {
		name: "Five Whys",
		description: "Root cause analysis chain",
		create: createFiveWhysTemplate,
	},
	pro_con: {
		name: "Pro/Con",
		description: "Decision analysis with pros and cons",
		create: createProConTemplate,
	},
	mece: {
		name: "MECE",
		description: "Mutually exclusive, collectively exhaustive breakdown",
		create: createMeceTemplate,
	},
} as const;

export type TemplateKey = keyof typeof TEMPLATES;
