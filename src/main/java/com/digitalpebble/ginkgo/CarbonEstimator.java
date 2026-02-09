// SPDX-License-Identifier: Apache-2.0

package com.digitalpebble.ginkgo;

import com.digitalpebble.ginkgo.model.ActionsBill;
import com.digitalpebble.ginkgo.model.UsageItem;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public class CarbonEstimator {

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    public static void main(String[] args) {
        try {
            // if an arg is passed, consider that it is a file that can be loaded locally
            ActionsBill actionsBill = null;

            String inputPath = null;
            if (args.length == 1) {
                inputPath = args[0];
                System.out.println("Loading billing data from file: " + inputPath);
                actionsBill = loadBillingUsage(Paths.get(inputPath));
                System.out.println("Loaded " + actionsBill.getUsageItems().size() + " usage items");
            } else {
                String token = getEnvRequired("INPUT_GITHUB-TOKEN");
                String organization = getEnvRequired("INPUT_ORGANIZATION");
                System.out.println("Fetching billing data for organization: " + organization);
                actionsBill = fetchBillingUsage(token, organization);
                System.out.println("Loaded " + actionsBill.getUsageItems().size() + " usage items");
            }

            System.out.println("Estimating carbon footprint...");
            calculateCarbonImpact(actionsBill);

            String outputPath = getEnv("OUTPUT-PATH", "./carbon-estimate.json");

            Path output = Paths.get(outputPath);
            Files.createDirectories(output.getParent());
            Files.writeString(output, actionsBill.toJson());

            System.out.println("Carbon estimate saved to: " + outputPath);

        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    static ActionsBill loadBillingUsage(Path path) throws IOException {
        return ActionsBill.fromFile(path);
    }

    private static ActionsBill fetchBillingUsage(String token, String organization) throws IOException, InterruptedException {
        HttpClient client = HttpClient.newHttpClient();

        String actionsUrl = String.format("https://api.github.com/orgs/%s/settings/billing/actions", organization);
        HttpRequest actionsRequest = HttpRequest.newBuilder()
                .uri(URI.create(actionsUrl))
                .header("Authorization", "Bearer " + token)
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .GET()
                .build();

        HttpResponse<String> actionsResponse = client.send(actionsRequest, HttpResponse.BodyHandlers.ofString());

        if (actionsResponse.statusCode() != 200) {
            throw new IOException("Failed to fetch Actions billing: " + actionsResponse.statusCode() + " - " + actionsResponse.body());
        }

        return ActionsBill.fromJson(actionsResponse.body());
    }

    static void calculateCarbonImpact(ActionsBill actionsBill) {
        Config config = Config.getInstance();
        double pue = config.getPue();
        double gridIntensity = config.getGridCarbonIntensity();

        for (UsageItem item : actionsBill.getUsageItems()) {
            if (!"Minutes".equals(item.getUnitType())) {
                continue;
            }

            double powerWatts = getRunnerPower(config, item.getSku());
            if (powerWatts == -1) {
                // TODO log unknown sku
                continue;
            }

            double hours = item.getQuantity() / 60.0;
            double energyWh = powerWatts * hours * pue;
            double co2eqG = energyWh / 1000.0 * gridIntensity;

            item.setEnergyUsageWh(Math.round(energyWh * 1000.0) / 1000.0);
            item.setCo2eqG(Math.round(co2eqG * 1000.0) / 1000.0);
        }
    }

    private static double getRunnerPower(Config config, String sku) {
        String runner = null;
        switch (sku) {
            case "Actions Linux ARM":
                runner = "ubuntu-arm";
                break;
            case "Actions Linux":
                runner = "ubuntu";
                break;
        }
        if (runner == null) {
            return -1d;
        }

        return config.getRunnerPowerConsumption().get(runner);
    }

    private static String getEnvRequired(String name) {
        String value = System.getenv(name);
        if (value == null || value.isEmpty()) {
            throw new IllegalArgumentException("Required environment variable not set: " + name);
        }
        return value;
    }

    private static String getEnv(String name, String defaultValue) {
        String value = System.getenv(name);
        return (value != null && !value.isEmpty()) ? value : defaultValue;
    }
}
