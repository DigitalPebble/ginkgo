FROM maven:3.9-eclipse-temurin-17 AS build

WORKDIR /app
COPY pom.xml .
COPY src ./src

RUN mvn clean package -DskipTests

FROM eclipse-temurin:17-jre-alpine

WORKDIR /action

COPY --from=build /app/target/ginkgo.jar /action/ginkgo.jar

ENTRYPOINT ["java", "-jar", "/action/ginkgo.jar"]
