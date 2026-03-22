import type { NodeTypes } from "@xyflow/react";
import ClaimNode from "./ClaimNode";
import CounterRebuttalNode from "./CounterRebuttalNode";
import EvidenceNode from "./EvidenceNode";
import RebuttalNode from "./RebuttalNode";

// MUST be defined at module scope — not inside a component.
// If defined inside a component, React Flow re-mounts all nodes on every render.
export const nodeTypes: NodeTypes = {
	claim: ClaimNode,
	evidence: EvidenceNode,
	rebuttal: RebuttalNode,
	counter_rebuttal: CounterRebuttalNode,
};
