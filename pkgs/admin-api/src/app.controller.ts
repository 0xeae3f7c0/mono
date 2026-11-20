// admin-api: Team and repository ownership API for the admin console (commit platform, pkgs/admin-api)
import { Body, Controller, Get, Param, Post } from '@nestjs/common';
import { AppService, type Team } from './app.service.js';

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Get('health')
  health() {
    return this.appService.health();
  }

  @Get('teams')
  teams(): Team[] {
    return this.appService.listTeams();
  }

  @Post('teams/:team/repos')
  assign(@Param('team') team: string, @Body() body: { repo: string }): Team {
    return this.appService.assign(team, body?.repo ?? '');
  }
}
