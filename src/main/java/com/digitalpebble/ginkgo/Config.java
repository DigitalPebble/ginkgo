// SPDX-License-Identifier: Apache-2.0

package com.digitalpebble.ginkgo;

import com.google.gson.Gson;
import com.google.gson.JsonObject;

import java.io.InputStreamReader;
import java.io.Reader;
import java.util.HashMap;
import java.util.Map;

public class Config {

    private static final Config INSTANCE = load();

    private final Map<String, Double> runnerPowerConsumption = new HashMap<>();
    private final double gridCarbonIntensity;
    private final double pue;

    private Config(JsonObject root) {
        JsonObject runners = root.getAsJsonObject("power_consumption")
                .getAsJsonObject("runners");
        for (String key : runners.keySet()) {
            runnerPowerConsumption.put(key, runners.getAsJsonObject(key).get("value").getAsDouble());
        }
        gridCarbonIntensity = root.getAsJsonObject("grid_carbon_intensity").get("value").getAsDouble();
        pue = root.getAsJsonObject("pue").get("value").getAsDouble();
    }

    private static Config load() {
        try (Reader reader = new InputStreamReader(
                Config.class.getClassLoader().getResourceAsStream("config.json"))) {
            JsonObject root = new Gson().fromJson(reader, JsonObject.class);
            return new Config(root);
        } catch (Exception e) {
            throw new RuntimeException("Failed to load config.json", e);
        }
    }

    public static Config getInstance() {
        return INSTANCE;
    }

    public Map<String, Double> getRunnerPowerConsumption() {
        return runnerPowerConsumption;
    }

    public double getGridCarbonIntensity() {
        return gridCarbonIntensity;
    }

    public double getPue() {
        return pue;
    }
}
