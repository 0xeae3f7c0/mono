import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import request from 'supertest';
import { App } from 'supertest/types';
import { AppModule } from './../src/app.module.js';

describe('admin-api (e2e)', () => {
  let app: INestApplication<App>;

  beforeEach(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({ imports: [AppModule] }).compile();
    app = moduleFixture.createNestApplication();
    await app.init();
  });

  afterEach(() => app.close());

  it('GET /health', () => request(app.getHttpServer()).get('/health').expect(200).expect({ ok: true }));

  it('POST then GET /teams', async () => {
    await request(app.getHttpServer()).post('/teams/platform/repos').send({ repo: 'o/api' }).expect(201);
    await request(app.getHttpServer()).get('/teams').expect(200).expect([{ name: 'platform', repos: ['o/api'] }]);
  });
});
