// Parallel Clarke & Wright savings heuristic (CWS) for the CVRP: C++ port of
// vrp.parallel_cws (src/vrp.py), function by function.
//
// Usage:  cws <data_dir> <output_csv>
// Reads <data_dir>/<name>_input_nodes.txt for the 33 instances (rows "x y demand", depot
// first), solves each one with the parallel CWS and writes one CSV row per instance with
// the number of routes, the cost and the mean time of one run in microseconds.
//
// Build (from the VRP folder):
//   uv run python -m ziglang c++ -std=c++20 -O2 -ffp-contract=off src/cpp/cws.cpp -o <exe>
// -ffp-contract=off keeps the floating-point operations exactly as in numpy, so savings,
// ties and costs are bit-for-bit the same as in the Python version.

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <string>
#include <utility>
#include <vector>

using Route = std::vector<int>;

struct Instance {
    std::vector<double> x, y, demand;
};

struct Edge {
    int i, j;
    double saving;
};

// Vehicle capacity of each instance (Juan et al. 2011, Table 1), same order as vrp.CAPACITY.
const std::vector<std::pair<std::string, double>> CAPACITY = {
    {"A-n32-k5", 100}, {"A-n38-k5", 100}, {"A-n45-k7", 100}, {"A-n55-k9", 100},
    {"A-n60-k9", 100}, {"A-n61-k9", 100}, {"A-n65-k9", 100}, {"A-n80-k10", 100},
    {"B-n50-k7", 100}, {"B-n52-k7", 100}, {"B-n57-k9", 100}, {"B-n78-k10", 100},
    {"E-n22-k4", 6000}, {"E-n30-k3", 4500}, {"E-n33-k4", 8000}, {"E-n51-k5", 160},
    {"E-n76-k7", 220}, {"E-n76-k10", 140}, {"E-n76-k14", 100},
    {"F-n45-k4", 2010}, {"F-n72-k4", 30000}, {"F-n135-k7", 2210},
    {"M-n101-k10", 200}, {"M-n121-k7", 200},
    {"P-n22-k8", 3000}, {"P-n40-k5", 140}, {"P-n50-k10", 100}, {"P-n55-k15", 70},
    {"P-n65-k10", 130}, {"P-n70-k10", 135}, {"P-n76-k4", 350}, {"P-n76-k5", 280},
    {"P-n101-k4", 400},
};

// Read coordinates and demands of an instance.
Instance load_instance(const std::string& data_dir, const std::string& name) {
    std::ifstream in(data_dir + "/" + name + "_input_nodes.txt");
    Instance inst;
    double x, y, q;
    while (in >> x >> y >> q) {
        inst.x.push_back(x);
        inst.y.push_back(y);
        inst.demand.push_back(q);
    }
    return inst;
}

// Euclidean distance between every pair of nodes, not rounded (flat n x n matrix).
std::vector<double> distance_matrix(const Instance& inst) {
    const int n = static_cast<int>(inst.x.size());
    std::vector<double> dist(n * n);
    for (int i = 0; i < n; ++i)
        for (int j = 0; j < n; ++j) {
            const double dx = inst.x[i] - inst.x[j], dy = inst.y[i] - inst.y[j];
            dist[i * n + j] = std::sqrt(dx * dx + dy * dy);
        }
    return dist;
}

// Customer pairs (i < j) sorted by decreasing savings s(i,j) = c0i + c0j - cij.
std::vector<Edge> make_savings_list(const std::vector<double>& dist, int n) {
    std::vector<Edge> edges;
    edges.reserve((n - 1) * (n - 2) / 2);
    for (int i = 1; i < n; ++i)
        for (int j = i + 1; j < n; ++j)
            edges.push_back({i, j, dist[i] + dist[j] - dist[i * n + j]});
    // ties keep the (i, j) generation order, as np.argsort(kind="stable") in Python
    std::stable_sort(edges.begin(), edges.end(),
                     [](const Edge& a, const Edge& b) { return a.saving > b.saving; });
    return edges;
}

// True if the node is adjacent to the depot, i.e. at one end of its route.
bool is_exterior(int node, const Route& route) {
    return route.front() == node || route.back() == node;
}

// CWS conditions: different routes, both nodes exterior and capacity respected.
bool check_merging_conditions(int i, int j, const std::vector<Route>& routes,
                              const std::vector<int>& route_of, const std::vector<double>& load,
                              double capacity) {
    const int ri = route_of[i], rj = route_of[j];
    return ri != rj && is_exterior(i, routes[ri]) && is_exterior(j, routes[rj]) &&
           load[ri] + load[rj] <= capacity;
}

// Join the route ending at i with the route starting at j; return the merged index.
int merge_routes_using_edge(int i, int j, std::vector<Route>& routes, std::vector<int>& route_of,
                            std::vector<double>& load) {
    const int ri = route_of[i], rj = route_of[j];
    Route& route_i = routes[ri];
    Route route_j = std::move(routes[rj]);
    routes[rj].clear();  // an empty slot is a route that no longer exists
    if (route_i.back() != i) std::reverse(route_i.begin(), route_i.end());
    if (route_j.front() != j) std::reverse(route_j.begin(), route_j.end());
    for (int c : route_j) route_of[c] = ri;
    route_i.insert(route_i.end(), route_j.begin(), route_j.end());
    load[ri] += load[rj];
    return ri;
}

// Parallel CWS: one pass over the savings list, any two routes may be merged.
std::vector<Route> parallel_cws(const std::vector<double>& dist, const std::vector<double>& demand,
                                double capacity) {
    const int n = static_cast<int>(demand.size());
    const std::vector<Edge> savings = make_savings_list(dist, n);
    // dummy solution: route i = [i] for every customer, same indices as in Python
    std::vector<Route> routes(n);
    std::vector<int> route_of(n);
    std::vector<double> load(demand);
    for (int i = 0; i < n; ++i) route_of[i] = i;
    for (int i = 1; i < n; ++i) routes[i] = {i};
    for (const Edge& e : savings)
        if (check_merging_conditions(e.i, e.j, routes, route_of, load, capacity))
            merge_routes_using_edge(e.i, e.j, routes, route_of, load);
    std::vector<Route> result;
    for (Route& r : routes)
        if (!r.empty()) result.push_back(std::move(r));
    return result;
}

// Length of the closed tour depot -> route -> depot.
double route_cost(const Route& route, const std::vector<double>& dist, int n) {
    double cost = dist[route.front()];
    for (size_t k = 0; k + 1 < route.size(); ++k) cost += dist[route[k] * n + route[k + 1]];
    return cost + dist[route.back()];
}

// Total length of all routes.
double solution_cost(const std::vector<Route>& routes, const std::vector<double>& dist, int n) {
    double cost = 0.0;
    for (const Route& r : routes) cost += route_cost(r, dist, n);
    return cost;
}

// True if every customer is visited exactly once and no route exceeds the capacity.
bool is_feasible(const std::vector<Route>& routes, const std::vector<double>& demand,
                 double capacity) {
    std::vector<int> visits(demand.size(), 0);
    for (const Route& r : routes) {
        double load = 0.0;
        for (int c : r) {
            ++visits[c];
            load += demand[c];
        }
        if (load > capacity) return false;
    }
    return std::all_of(visits.begin() + 1, visits.end(), [](int v) { return v == 1; });
}

int main(int argc, char** argv) {
    if (argc != 3) {
        std::cerr << "usage: cws <data_dir> <output_csv>\n";
        return 1;
    }
    const std::string data_dir = argv[1];
    std::ofstream out(argv[2]);
    out << "instance,routes,cost,time_us\n";
    for (const auto& [name, capacity] : CAPACITY) {
        const Instance inst = load_instance(data_dir, name);
        const int n = static_cast<int>(inst.demand.size());
        if (n == 0) {
            std::cerr << "cannot read " << name << "\n";
            return 1;
        }
        const std::vector<double> dist = distance_matrix(inst);
        // repeat for at least 0.1 s so that the mean time of one run is meaningful
        using clock = std::chrono::steady_clock;
        std::vector<Route> routes;
        int reps = 0;
        const auto t0 = clock::now();
        do {
            routes = parallel_cws(dist, inst.demand, capacity);
            ++reps;
        } while (clock::now() - t0 < std::chrono::milliseconds(100));
        const double us = std::chrono::duration<double, std::micro>(clock::now() - t0).count() / reps;
        if (!is_feasible(routes, inst.demand, capacity)) {
            std::cerr << name << ": infeasible solution\n";
            return 1;
        }
        const double cost = solution_cost(routes, dist, n);
        char row[160];
        std::snprintf(row, sizeof row, "%s,%zu,%.6f,%.2f\n", name.c_str(), routes.size(), cost, us);
        out << row;
        std::printf("%-11s routes=%3zu cost=%9.2f time=%8.2f us\n", name.c_str(), routes.size(), cost, us);
    }
    return 0;
}
