from pathlib import Path
v={r.split(':')[0]:int(r.split()[1]) for r in Path('/proc/meminfo').read_text().splitlines() if len(r.split())>1 and r.split()[1].isdigit()}
used=v['MemTotal']-v['MemAvailable'];ready=used<=14*1024*1024
print(f'RAM before next config: used={used/1024/1024:.2f}GiB; '+('ready' if ready else 'holding (14GiB threshold)'))
raise SystemExit(0 if ready else 1)
