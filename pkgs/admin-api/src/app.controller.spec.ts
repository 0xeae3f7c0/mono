// admin-api: Team and repository ownership API for the admin console (commit platform, pkgs/admin-api)
import { Test, TestingModule } from '@nestjs/testing';
import { BadRequestException } from '@nestjs/common';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';

describe('AppController', () => {
  let controller: AppController;

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
      providers: [AppService],
    }).compile();
    controller = app.get<AppController>(AppController);
  });

  it('reports health', () => {
    expect(controller.health()).toEqual({ ok: true });
  });

  it('assigns repositories to teams without duplicates', () => {
    controller.assign('platform', { repo: 'o/api' });
    controller.assign('platform', { repo: 'o/api' });
    controller.assign('apps', { repo: 'o/web' });
    expect(controller.teams()).toEqual([
      { name: 'apps', repos: ['o/web'] },
      { name: 'platform', repos: ['o/api'] },
    ]);
  });

  it('validates input', () => {
    expect(() => controller.assign('Bad Team', { repo: 'o/r' })).toThrow(BadRequestException);
    expect(() => controller.assign('ok', { repo: 'nope' })).toThrow(BadRequestException);
  });
});
