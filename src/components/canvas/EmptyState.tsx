export default function EmptyState() {
	return (
		<div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
			<div className="text-center">
				{/* Map icon as SVG */}
				<svg
					className="mx-auto h-16 w-16 text-zinc-700"
					fill="none"
					stroke="currentColor"
					strokeWidth={1.5}
					viewBox="0 0 24 24"
				>
					<path
						strokeLinecap="round"
						strokeLinejoin="round"
						d="M9 6.75V15m6-6v8.25m.503-8.914l3.75-1.607a.75.75 0 01.997.707v7.628a.75.75 0 01-.497.707l-3.75 1.607M9 6.75L6.253 5.143a.75.75 0 00-.997.707v7.628a.75.75 0 00.497.707L9 15.75m0-9l6-2.25m0 0l3.75 1.607M15 3.75L9 6"
					/>
				</svg>
				<p className="mt-4 text-lg font-light text-zinc-500">
					Start by adding a Claim
				</p>
				<p className="mt-1 text-sm text-zinc-600">
					Press C or click + Claim in the sidebar
				</p>
			</div>
		</div>
	);
}
