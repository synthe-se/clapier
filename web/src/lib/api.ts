// The burrow's JSON (crates/clapier/src/lib.rs): one status answer, three gestures. A refusal
// carries a code the app words itself (errors.ts).

export interface Rabbit {
	mac: string | null;
	/** What the person called it (kept in its burrow); null until named. */
	name: string | null;
	ip: string;
	version: string | null;
	boot_s: number | null;
	pulse_s: number | null;
	status_s: number | null;
	uptime_s: number | null;
	link: number | null;
	rssi: number | null;
	ears: string | null;
	audio: string | null;
	choreo: string | null;
	reassoc: number | null;
	tcp_retx: number | null;
	/** The nabgcc build it runs on; null when its firmware does not say. */
	firmware: string | null;
	/** How its last firmware fetch ended, in its own words; null before the first one. */
	fw_last: string | null;
}

/** A firmware image the burrow holds for its rabbits. */
export interface Firmware {
	name: string;
	bytes: number;
}

export interface Hit {
	ago_ms: number;
	peer: string;
	method: string;
	uri: string;
	status: number;
	bytes: number;
	rabbit: boolean;
}

export interface Status {
	version: string;
	uptime_s: number;
	serving: string;
	tagged: { ip: string; seen_s: number | null } | null;
	rabbits: Rabbit[];
	chors: string[];
	firmwares: Firmware[];
	/** The MP3s a rabbit can stream, as /vl paths. */
	sounds: string[];
	journal: Hit[];
}

export interface Reply {
	reply: string | null;
	bytes?: number;
}

/** A refusal from the burrow (`code`), or `unreachable` when it did not answer at all. */
export class ApiError extends Error {
	constructor(
		readonly code: string,
		readonly detail?: string
	) {
		super(detail ? `${code}: ${detail}` : code);
	}
}

const BASE = '/api';

async function call<T>(path: string, init?: RequestInit): Promise<T> {
	let res: Response;
	try {
		res = await fetch(`${BASE}/${path}`, init);
	} catch {
		throw new ApiError('unreachable');
	}
	if (!res.ok) {
		const body = await res.json().catch(() => null);
		throw new ApiError(typeof body?.code === 'string' ? body.code : 'unreachable', body?.detail);
	}
	return res.status === 202 || res.status === 204 ? (undefined as T) : res.json();
}

const post = <T>(path: string, body: unknown) =>
	call<T>(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });

export const api = {
	status: () => call<Status>('status'),
	ctl: (ip: string, cmd: string) => post<Reply>('ctl', { ip, cmd }),
	dance: (mac: string, ip: string, cdl: string) => post<Reply>('dance', { mac, ip, cdl }),
	name: (mac: string, name: string) => post<void>('name', { mac, name }),
	/** Hands a rabbit one of the burrow's images: to check it only, or to flash it. */
	firmware: (ip: string, name: string, flash: boolean) => post<Reply>('firmware', { ip, name, flash }),
	/** Keeps a .sim (or a raw image) in the burrow; answers the name it is kept under. */
	/** Keeps an MP3 in the burrow; answers the /vl path it is served at. */
	keepSound: (file: File) =>
		call<Reply>(`sound/${encodeURIComponent(file.name)}`, { method: 'PUT', headers: { 'content-type': 'application/octet-stream' }, body: file }),
	keepFirmware: (file: File) =>
		call<Reply>(`firmware/${encodeURIComponent(file.name)}`, { method: 'PUT', headers: { 'content-type': 'application/octet-stream' }, body: file })
};
