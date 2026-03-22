import { invoke } from "@tauri-apps/api/core";
import type {
	ArgEdge,
	ArgMap,
	ArgNode,
	EdgePayload,
	NodePayload,
} from "../types";

export const tauriApi = {
	getMaps: () => invoke<ArgMap[]>("get_maps"),

	createMap: (title: string, description?: string) =>
		invoke<ArgMap>("create_map", { title, description }),

	deleteMap: (mapId: string) => invoke<void>("delete_map", { mapId }),

	renameMap: (mapId: string, title: string) =>
		invoke<void>("rename_map", { mapId, title }),

	loadMap: (mapId: string) =>
		invoke<{ nodes: ArgNode[]; edges: ArgEdge[] }>("load_map", { mapId }),

	saveMapState: (mapId: string, nodes: NodePayload[], edges: EdgePayload[]) =>
		invoke<void>("save_map_state", { mapId, nodes, edges }),

	exportMapJson: (mapId: string) =>
		invoke<string>("export_map_json", { mapId }),
};
