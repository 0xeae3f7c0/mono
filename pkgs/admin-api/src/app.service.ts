import { BadRequestException, Injectable } from '@nestjs/common';

/** A team in the admin console and the repositories it owns. */
export interface Team {
  name: string;
  repos: string[];
}

/**
 * Backend for the Angular admin console: manages which team owns which
 * tracked repositories (in memory here; repo-registry persists the list).
 */
@Injectable()
export class AppService {
  private readonly teams = new Map<string, Team>();

  health() {
    return { ok: true };
  }

  listTeams(): Team[] {
    return [...this.teams.values()].sort((a, b) => a.name.localeCompare(b.name));
  }

  assign(team: string, repo: string): Team {
    if (!/^[a-z0-9-]+$/.test(team)) throw new BadRequestException('team must be lowercase letters, digits or -');
    if (!/^[\w.-]+\/[\w.-]+$/.test(repo)) throw new BadRequestException('repo must look like owner/name');
    const entry = this.teams.get(team) ?? { name: team, repos: [] };
    if (!entry.repos.includes(repo)) entry.repos.push(repo);
    this.teams.set(team, entry);
    return entry;
  }
}
