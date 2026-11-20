// portal: Developer portal cataloguing every service here (commit platform, pkgs/portal)
import { createApp } from '@backstage/frontend-defaults';
import catalogPlugin from '@backstage/plugin-catalog/alpha';
import { navModule } from './modules/nav';
import { homeModule } from './modules/home';

export default createApp({
  features: [catalogPlugin, navModule, homeModule],
});
