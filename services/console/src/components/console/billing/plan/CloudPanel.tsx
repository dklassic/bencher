import bencher_valid_init, { type InitOutput } from "bencher_valid";
import { Match, Switch, createMemo, createResource } from "solid-js";
import consoleConfig from "../../../../config/console";
import { Operation, BencherResource } from "../../../../config/types";
import type { Params } from "astro";
import { Host } from "../../../../config/organization/billing";
import { authUser } from "../../../../util/auth";
import { isBencherCloud } from "../../../../util/ext";
import BillingHeader, { BillingHeaderConfig } from "../BillingHeader";
import SelfHostedPlan from "./SelfHostedPanel";
import type { JsonPlan } from "../../../../types/bencher";
import { httpGet } from "../../../../util/http";
import { validJwt } from "../../../../util/valid";
import BillingForm from "./BillingForm";
import Plan from "./Plan";

interface Props {
	apiUrl: string;
	params: Params;
}

interface CloudPanelConfig {
	operation: Operation;
	header: BillingHeaderConfig;
	host: Host;
}

const CloudPanel = (props: Props) => {
	const [bencher_valid] = createResource(
		async () => await bencher_valid_init(),
	);
	const user = authUser();
	const host = createMemo(() =>
		isBencherCloud() ? Host.BENCHER_CLOUD : Host.SELF_HOSTED,
	);
	const config = createMemo<CloudPanelConfig>(
		() => consoleConfig[BencherResource.BILLING]?.[host()],
	);

	const fetcher = createMemo(() => {
		return {
			params: props.params,
			bencher_valid: bencher_valid(),
			token: user?.token,
		};
	});
	const fetchPlan = async (fetcher: {
		params: Params;
		bencher_valid: InitOutput;
		token: string;
	}) => {
		if (!fetcher.bencher_valid) {
			return null;
		}
		if (!validJwt(fetcher.token)) {
			return null;
		}
		const path = `/v0/organizations/${fetcher.params.organization}/plan`;
		return await httpGet(props.apiUrl, path, fetcher.token)
			.then((resp) => {
				return resp?.data;
			})
			.catch((_error) => {
				return null;
			});
	};
	const [plan, { refetch }] = createResource<null | JsonPlan>(
		fetcher,
		fetchPlan,
	);

	return (
		<>
			<BillingHeader config={config()?.header} />

			<Switch
				fallback={
					<section class="section">
						<div class="container">
							<h4 class="title">Loading...</h4>
						</div>
					</section>
				}
			>
				<Match when={config()?.host === Host.SELF_HOSTED}>
					<SelfHostedPlan
						apiUrl={props.apiUrl}
						params={props.params}
						bencher_valid={bencher_valid}
						user={user}
					/>
				</Match>
				<Match when={config()?.host === Host.BENCHER_CLOUD && plan() === null}>
					<BillingForm
						apiUrl={props.apiUrl}
						params={props.params}
						bencher_valid={bencher_valid}
						user={user}
						handleRefresh={refetch}
					/>
				</Match>
				<Match when={config()?.host === Host.BENCHER_CLOUD && plan()}>
					<Plan
						apiUrl={props.apiUrl}
						params={props.params}
						bencher_valid={bencher_valid}
						user={user}
						plan={plan}
					/>
				</Match>
			</Switch>
		</>
	);
};

export default CloudPanel;