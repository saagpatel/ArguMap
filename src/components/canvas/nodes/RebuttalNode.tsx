import type { Node, NodeProps } from "@xyflow/react";
import type { ArgNodeData } from "../../../types";
import BaseNode from "./BaseNode";

export default function RebuttalNode(props: NodeProps<Node<ArgNodeData>>) {
	return <BaseNode {...props} />;
}
