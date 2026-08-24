import { useEffect, useState } from "react";
import { tauriApi } from "../../lib/tauri";
import type { ResearchProjection } from "../../types";

const MAX_PACKAGE_BYTES = 5 * 1024 * 1024;

interface ResearchExchangePanelProps {
	mapId: string | null;
	onImported: () => void;
}

function downloadCanonical(contents: string, projection: ResearchProjection) {
	const blob = new Blob([contents], { type: "application/json" });
	const url = URL.createObjectURL(blob);
	const anchor = document.createElement("a");
	anchor.href = url;
	anchor.download = `${projection.package_id}-${projection.revision_id}.research.json`;
	anchor.click();
	URL.revokeObjectURL(url);
}

export default function ResearchExchangePanel({
	mapId,
	onImported,
}: ResearchExchangePanelProps) {
	const [raw, setRaw] = useState("");
	const [projection, setProjection] = useState<ResearchProjection | null>(null);
	const [error, setError] = useState("");
	const [busy, setBusy] = useState(false);
	const [persisted, setPersisted] = useState(false);

	useEffect(() => {
		let active = true;
		setProjection(null);
		setPersisted(false);
		setRaw("");
		setError("");
		if (!mapId)
			return () => {
				active = false;
			};
		void (async () => {
			try {
				const retained = await tauriApi.loadPersistedResearchPackage(mapId);
				if (!active || !retained) return;
				const canonical = await tauriApi.exportPersistedCanonicalResearchPackage(mapId);
				if (!active) return;
				setProjection(retained);
				setRaw(canonical);
				setPersisted(true);
			} catch (reason) {
				if (active) setError(reason instanceof Error ? reason.message : String(reason));
			}
		})();
		return () => {
			active = false;
		};
	}, [mapId]);

	const run = async (mode: "inspect" | "import") => {
		if (!mapId) return;
		setBusy(true);
		setError("");
		try {
			const result =
				mode === "inspect"
					? await tauriApi.inspectResearchPackage(raw, mapId)
					: await tauriApi.importResearchPackageIntoMap(raw, mapId);
			setProjection(result);
			setPersisted(mode === "import");
			if (mode === "import") onImported();
		} catch (reason) {
			setProjection(null);
			setError(reason instanceof Error ? reason.message : String(reason));
		} finally {
			setBusy(false);
		}
	};

	return (
		<details className="border-t border-zinc-800 pt-3">
			<summary className="cursor-pointer text-xs font-medium uppercase tracking-wide text-zinc-400">
				Research exchange
			</summary>
			<div className="mt-3 space-y-3">
				<p className="text-xs leading-relaxed text-zinc-500">
					Inspect or project reviewed P0/P1 research JSON. Non-native lifecycle,
					method, and conclusion semantics stay in the canonical package and are
					reported as losses.
				</p>
				<label className="block text-xs text-zinc-400">
					Package file
					<input
						className="mt-1 block w-full text-xs text-zinc-500"
						type="file"
						accept="application/json,.json"
						onChange={async (event) => {
							const file = event.target.files?.[0];
							if (!file) return;
							if (file.size > MAX_PACKAGE_BYTES) {
								setError("Research package exceeds the 5 MiB limit.");
								return;
							}
							setRaw(await file.text());
							setProjection(null);
							setPersisted(false);
							setError("");
						}}
					/>
				</label>
				<label className="block text-xs text-zinc-400">
					Package JSON
					<textarea
						className="mt-1 min-h-28 w-full resize-y rounded border border-zinc-700 bg-zinc-950 p-2 font-mono text-[10px] text-zinc-300 outline-none focus:border-blue-500"
						value={raw}
						onChange={(event) => {
							setRaw(event.target.value);
							setProjection(null);
							setPersisted(false);
						}}
						placeholder='{"schema_version":"evidence-centered.research-package.v2", ...}'
						spellCheck={false}
					/>
				</label>
				<div className="grid grid-cols-2 gap-2">
					<button
						type="button"
						className="rounded border border-zinc-700 px-2 py-1.5 text-xs text-zinc-300 disabled:opacity-40"
						disabled={!mapId || !raw.trim() || busy}
						onClick={() => run("inspect")}
					>
						Inspect
					</button>
					<button
						type="button"
						className="rounded bg-blue-600 px-2 py-1.5 text-xs text-white disabled:opacity-40"
						disabled={!mapId || !raw.trim() || busy}
						onClick={() => run("import")}
					>
						Project into map
					</button>
				</div>
				{projection && (
					<div className="space-y-2 rounded border border-zinc-800 bg-zinc-950 p-2 text-[10px] text-zinc-400">
						<p className="break-all text-zinc-300">{projection.package_id}</p>
						<p>
							{projection.nodes.length} nodes · {projection.edges.length} edges ·{" "}
							{projection.losses.length} explicit losses
						</p>
						<p className="break-all font-mono">{projection.schema_digest}</p>
						<button
							type="button"
							className="text-blue-400 hover:text-blue-300"
							onClick={async () => {
								if (!mapId) return;
								try {
									const canonical = persisted
										? await tauriApi.exportPersistedCanonicalResearchPackage(mapId)
										: await tauriApi.exportCanonicalResearchPackage(raw, mapId);
									downloadCanonical(canonical, projection);
								} catch (reason) {
									setError(reason instanceof Error ? reason.message : String(reason));
								}
							}}
						>
							Export retained canonical JSON
						</button>
					</div>
				)}
				{error && (
					<p className="text-xs text-red-400" role="alert">
						{error}
					</p>
				)}
			</div>
		</details>
	);
}
