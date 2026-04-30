import { createRootRoute, createRoute, Outlet } from '@tanstack/react-router';
import { z } from 'zod';
import { Layout } from './components/Layout';
import { HomePage } from './pages/HomePage';
import { TxPage } from './pages/TxPage';
import { AddressPage } from './pages/AddressPage';
import { PackagePage } from './pages/PackagePage';
import { CheckpointPage } from './pages/CheckpointPage';
import { CheckpointsPage } from './pages/CheckpointsPage';
import { PackagesPage } from './pages/PackagesPage';
import { SecurityPage } from './pages/SecurityPage';
import { NetworkPage } from './pages/NetworkPage';
import { SearchPage } from './pages/SearchPage';
import { LabelsPage } from './pages/LabelsPage';
import { WatchlistPage } from './pages/WatchlistPage';
import { AlertsPage } from './pages/AlertsPage';
import { LoginPage } from './pages/LoginPage';
import { DeploymentsPage } from './pages/DeploymentsPage';
import { ActivePage } from './pages/ActivePage';
import { TvlPage } from './pages/TvlPage';
import { WatchPage } from './pages/WatchPage';
import { WatchProtocolPage } from './pages/WatchProtocolPage';

const rootRoute = createRootRoute({
  component: () => (
    <Layout>
      <Outlet />
    </Layout>
  ),
});

const indexRoute = createRoute({ getParentRoute: () => rootRoute, path: '/', component: HomePage });

const txRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tx/$digest',
  component: TxPage,
});

const addressRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/address/$addr',
  component: AddressPage,
});

const packageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/package/$id',
  component: PackagePage,
});

const checkpointRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/checkpoint/$seq',
  component: CheckpointPage,
});

const checkpointsListRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/checkpoints',
  component: CheckpointsPage,
});

const packagesListRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/packages',
  component: PackagesPage,
});

const securityRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/security',
  component: SecurityPage,
});

const networkRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/network',
  component: NetworkPage,
});

const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/search',
  validateSearch: (search) => z.object({ q: z.string().default('') }).parse(search),
  component: SearchPage,
});

const labelsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/labels',
  component: LabelsPage,
});

const watchlistRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/watchlist',
  component: WatchlistPage,
});

const alertsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/alerts',
  component: AlertsPage,
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
});

const deploymentsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/analytics/deployments',
  component: DeploymentsPage,
});

const activeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/analytics/active',
  component: ActivePage,
});

const tvlRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/analytics/tvl',
  component: TvlPage,
});

const watchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/watch',
  component: WatchPage,
});

const watchDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/watch/$id',
  component: WatchProtocolPage,
});

export const routeTree = rootRoute.addChildren([
  indexRoute,
  txRoute,
  addressRoute,
  packageRoute,
  checkpointRoute,
  checkpointsListRoute,
  packagesListRoute,
  securityRoute,
  networkRoute,
  searchRoute,
  labelsRoute,
  watchlistRoute,
  alertsRoute,
  loginRoute,
  deploymentsRoute,
  activeRoute,
  tvlRoute,
  watchRoute,
  watchDetailRoute,
]);
