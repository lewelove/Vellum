def generate_report_text(year, matched, unknown, fuzzy_debug, alb_min, trk_min):
    """Formats the final report string."""
    lines = []
    lines.append(f"{year if year else 'TOTAL'} REPORT")
    lines.append("")
    
    lines.append("  Most Listened Albums")
    filtered_matched = [m for m in matched if m["listens"] >= alb_min]
    filtered_matched.sort(key=lambda x: x["listens"], reverse=True)
    for m in filtered_matched:
        lines.append(f"    {m['artist']} - {m['album']}: {m['listens']}{m['suffix']}")
        
    lines.append("")
    lines.append("  Total Track Listens from Unknown Albums")
    filtered_unknown = [u for u in unknown if u["listens"] >= trk_min]
    filtered_unknown.sort(key=lambda x: x["listens"], reverse=True)
    for u in filtered_unknown:
        lines.append(f"    {u['artist']} - {u['album']}: {u['listens']}")

    if fuzzy_debug:
        lines.append("")
        lines.append("  Fuzzy Found Matches (Debug)")
        fuzzy_debug.sort(key=lambda x: x["score"], reverse=True)
        for f in fuzzy_debug[:15]:
            lines.append(f"    {f['lb_tuple']}: {f['lib_tuple']} - {f['score']}")
        
    return "\n".join(lines)
