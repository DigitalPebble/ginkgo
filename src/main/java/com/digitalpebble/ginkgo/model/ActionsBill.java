package com.digitalpebble.ginkgo.model;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

public class ActionsBill {

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    private List<UsageItem> usageItems = new ArrayList<>();

    public List<UsageItem> getUsageItems() {
        return usageItems;
    }

    public void setUsageItems(List<UsageItem> usageItems) {
        this.usageItems = usageItems;
    }

    public static ActionsBill fromJson(String json) {
        return GSON.fromJson(json, ActionsBill.class);
    }

    public static ActionsBill fromFile(Path path) throws IOException {
        try (Reader reader = Files.newBufferedReader(path)) {
            return GSON.fromJson(reader, ActionsBill.class);
        }
    }

    public String toJson() {
        return GSON.toJson(this);
    }

    public void toFile(Path path) throws IOException {
        try (Writer writer = Files.newBufferedWriter(path)) {
            GSON.toJson(this, writer);
        }
    }
}
