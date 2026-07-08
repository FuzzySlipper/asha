import { manifestDiagnostic, } from './manifest-types.js';
export function parseTomlSubset(toml) {
    const document = {};
    let currentSection = null;
    const diagnostics = [];
    toml.split(/\r?\n/).forEach((rawLine, index) => {
        const lineNumber = index + 1;
        const line = stripComment(rawLine).trim();
        if (line.length === 0) {
            return;
        }
        const sectionMatch = /^\[([A-Za-z0-9_-]+)\]$/.exec(line);
        if (sectionMatch) {
            currentSection = sectionMatch[1];
            document[currentSection] ??= {};
            return;
        }
        if (currentSection === null) {
            diagnostics.push(manifestDiagnostic('toml_parse_error', `line ${lineNumber}`, 'manifest keys must be inside a section'));
            return;
        }
        const assignmentMatch = /^([A-Za-z0-9_]+)\s*=\s*(.+)$/.exec(line);
        if (!assignmentMatch) {
            diagnostics.push(manifestDiagnostic('toml_parse_error', `line ${lineNumber}`, 'expected key = value'));
            return;
        }
        const key = assignmentMatch[1];
        const rawValue = assignmentMatch[2].trim();
        const value = parseTomlValue(rawValue, `line ${lineNumber}`);
        if (value.ok) {
            document[currentSection][key] = value.value;
        }
        else {
            diagnostics.push(value.diagnostic);
        }
    });
    return diagnostics.length === 0 ? { ok: true, document } : { ok: false, diagnostics };
}
function stripComment(line) {
    let inString = false;
    for (let i = 0; i < line.length; i += 1) {
        const char = line[i];
        if (char === '"' && line[i - 1] !== '\\') {
            inString = !inString;
        }
        if (char === '#' && !inString) {
            return line.slice(0, i);
        }
    }
    return line;
}
function parseTomlValue(rawValue, path) {
    if (rawValue === 'true') {
        return { ok: true, value: true };
    }
    if (rawValue === 'false') {
        return { ok: true, value: false };
    }
    if (rawValue.startsWith('"') && rawValue.endsWith('"')) {
        return { ok: true, value: rawValue.slice(1, -1) };
    }
    if (rawValue.startsWith('[') && rawValue.endsWith(']')) {
        const inner = rawValue.slice(1, -1).trim();
        if (inner.length === 0) {
            return { ok: true, value: [] };
        }
        const values = inner.split(',').map((part) => part.trim());
        if (!values.every((part) => part.startsWith('"') && part.endsWith('"'))) {
            return {
                ok: false,
                diagnostic: manifestDiagnostic('toml_parse_error', path, 'only string arrays are supported in asha.game.toml'),
            };
        }
        return { ok: true, value: values.map((part) => part.slice(1, -1)) };
    }
    return {
        ok: false,
        diagnostic: manifestDiagnostic('toml_parse_error', path, 'expected a string, boolean, or string array'),
    };
}
//# sourceMappingURL=manifest-toml.js.map