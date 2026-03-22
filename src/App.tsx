import { ReactFlowProvider } from "@xyflow/react";
import { useCallback, useEffect, useRef, useState } from "react";
import ArgCanvas from "./components/canvas/ArgCanvas";
import { tauriApi } from "./lib/tauri";
import type { ArgMap } from "./types";

function App() {
	const [activeMapId, setActiveMapId] = useState<string | null>(null);
	const [mapTitle, setMapTitle] = useState("ArguMap Studio");
	const [maps, setMaps] = useState<ArgMap[]>([]);
	const [editingTitle, setEditingTitle] = useState(false);
	const titleInputRef = useRef<HTMLInputElement>(null);

	const refreshMaps = useCallback(() => {
		tauriApi.getMaps().then(setMaps).catch(console.error);
	}, []);

	// Startup: load most recent map or create one
	useEffect(() => {
		let cancelled = false;

		tauriApi
			.getMaps()
			.then((loadedMaps) => {
				if (cancelled) return;
				setMaps(loadedMaps);
				if (loadedMaps.length > 0) {
					setActiveMapId(loadedMaps[0].id);
					setMapTitle(loadedMaps[0].title);
				} else {
					return tauriApi.createMap("Untitled Map").then((newMap) => {
						if (cancelled) return;
						setActiveMapId(newMap.id);
						setMapTitle(newMap.title);
						refreshMaps();
					});
				}
			})
			.catch((err: unknown) => {
				console.error("Failed to initialize:", err);
			});

		return () => {
			cancelled = true;
		};
	}, [refreshMaps]);

	// Map management callbacks
	const handleSelectMap = useCallback(
		(mapId: string) => {
			const map = maps.find((m) => m.id === mapId);
			if (map) {
				setActiveMapId(mapId);
				setMapTitle(map.title);
			}
		},
		[maps],
	);

	const handleCreateMap = useCallback(() => {
		tauriApi
			.createMap("Untitled Map")
			.then((newMap) => {
				setActiveMapId(newMap.id);
				setMapTitle(newMap.title);
				refreshMaps();
			})
			.catch(console.error);
	}, [refreshMaps]);

	const handleRenameMap = useCallback(
		(mapId: string, title: string) => {
			tauriApi
				.renameMap(mapId, title)
				.then(() => {
					if (mapId === activeMapId) setMapTitle(title);
					refreshMaps();
				})
				.catch(console.error);
		},
		[activeMapId, refreshMaps],
	);

	const handleDeleteMap = useCallback(
		(mapId: string) => {
			tauriApi
				.deleteMap(mapId)
				.then(() => {
					if (mapId === activeMapId) {
						tauriApi.getMaps().then((remaining) => {
							if (remaining.length > 0) {
								setActiveMapId(remaining[0].id);
								setMapTitle(remaining[0].title);
							} else {
								tauriApi.createMap("Untitled Map").then((newMap) => {
									setActiveMapId(newMap.id);
									setMapTitle(newMap.title);
									refreshMaps();
								});
							}
							setMaps(remaining);
						});
					} else {
						refreshMaps();
					}
				})
				.catch(console.error);
		},
		[activeMapId, refreshMaps],
	);

	// Inline title rename
	const handleTitleClick = useCallback(() => {
		setEditingTitle(true);
		setTimeout(() => titleInputRef.current?.select(), 0);
	}, []);

	const handleTitleSubmit = useCallback(
		(newTitle: string) => {
			const trimmed = newTitle.trim();
			if (trimmed && activeMapId) {
				handleRenameMap(activeMapId, trimmed);
			}
			setEditingTitle(false);
		},
		[activeMapId, handleRenameMap],
	);

	return (
		<ReactFlowProvider>
			<div className="flex h-screen flex-col bg-[#0F0F0F]">
				<header className="flex h-10 shrink-0 items-center border-b border-zinc-800 bg-[#111111] px-4">
					{editingTitle ? (
						<input
							ref={titleInputRef}
							className="bg-transparent text-sm font-light tracking-wide text-zinc-300 outline-none"
							defaultValue={mapTitle}
							onBlur={(e) => handleTitleSubmit(e.target.value)}
							onKeyDown={(e) => {
								if (e.key === "Enter") handleTitleSubmit(e.currentTarget.value);
								if (e.key === "Escape") setEditingTitle(false);
							}}
							autoFocus
						/>
					) : (
						<span
							className="cursor-pointer text-sm font-light tracking-wide text-zinc-300 hover:text-zinc-100"
							onClick={handleTitleClick}
						>
							{mapTitle}
						</span>
					)}
				</header>
				<div className="flex-1 overflow-hidden">
					{activeMapId && (
						<ArgCanvas
							mapId={activeMapId}
							mapTitle={mapTitle}
							maps={maps}
							activeMapId={activeMapId}
							onSelectMap={handleSelectMap}
							onCreateMap={handleCreateMap}
							onRenameMap={handleRenameMap}
							onDeleteMap={handleDeleteMap}
						/>
					)}
				</div>
			</div>
		</ReactFlowProvider>
	);
}

export default App;
