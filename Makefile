viz:
	@firefox data/04_visualization/risk_return.html

viz_hull:
	@firefox data/04_visualization/convex_hull.html

run:
	@cargo run -r

run_store_logs:
	@cargo run -r 2>data/06_logs/log.info
