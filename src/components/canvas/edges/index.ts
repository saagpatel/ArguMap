import type { EdgeTypes } from "@xyflow/react";
import TypedEdge from "./TypedEdge";

// Module scope — React Flow uses 'default' when edge has no explicit type
export const edgeTypes: EdgeTypes = {
	default: TypedEdge,
};
