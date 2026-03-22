import { toPng } from "html-to-image";
import { tauriApi } from "./tauri";

export async function exportAsPng(mapTitle: string): Promise<void> {
	const viewport = document.querySelector(
		".react-flow__viewport",
	) as HTMLElement;
	if (!viewport) return;

	const dataUrl = await toPng(viewport, {
		backgroundColor: "#0A0A0A",
		pixelRatio: 2,
	});

	const link = document.createElement("a");
	link.href = dataUrl;
	link.download = `${mapTitle}.png`;
	link.click();
}

export async function exportAsJson(
	mapId: string,
	mapTitle: string,
): Promise<void> {
	const json = await tauriApi.exportMapJson(mapId);
	const blob = new Blob([json], { type: "application/json" });
	const url = URL.createObjectURL(blob);

	const link = document.createElement("a");
	link.href = url;
	link.download = `${mapTitle}.json`;
	link.click();
	URL.revokeObjectURL(url);
}
